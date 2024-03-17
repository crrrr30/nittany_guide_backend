use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionResponseFormat, ChatCompletionResponseFormatType,
        CreateChatCompletionRequest, Role,
    },
    Client,
};
use axum::{
    extract::{self, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use hex::ToHex;
use serde::{Deserialize, Serialize};
use state::AppState;
use tower_http::cors::CorsLayer;

use crate::db::{DocumentId, DocumentRecord};

mod db;
mod error;
mod state;

use error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonError {
    pub message: String,
}

pub struct JsonErrorResponse(StatusCode, String);

impl IntoResponse for JsonErrorResponse {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(JsonError {
                message: self.1.into(),
            }),
        )
            .into_response()
    }
}

impl From<Error> for JsonErrorResponse {
    fn from(e: Error) -> Self {
        JsonErrorResponse(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error".into(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    id: String,
}

async fn upload(
    state: Extension<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, JsonErrorResponse> {
    println!("Connection...");
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "what-if" {
            let data = field.bytes().await.unwrap();
            println!("Length of `{}` is {} bytes", name, data.len());

            let output = pdf_extract::extract_text_from_mem(&data).unwrap();

            let key = state.db.insert_document(&DocumentRecord {
                content: output,
                created: Utc::now(),
            })?;

            let key: String = key.0.encode_hex();

            return Ok(Json(UploadResponse { id: key }));
        } else {
            return Err(JsonErrorResponse(
                StatusCode::NOT_FOUND,
                String::from("bad frontend"),
            ));
        }
    }
    return Err(JsonErrorResponse(
        StatusCode::NOT_FOUND,
        String::from("bad frontend"),
    ));
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateRecommend {
    id: String,
    major: String,
    campus: String,
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecommendResponse {}

async fn recommend(
    state: Extension<AppState>,
    body: extract::Json<CreateRecommend>,
) -> Result<Json<serde_json::Value>, JsonErrorResponse> {
    let ref client = state.client;
    let mut request = CreateChatCompletionRequest::default();
    request.response_format = Some(ChatCompletionResponseFormat {
        r#type: ChatCompletionResponseFormatType::JsonObject,
    });
    let id = hex::decode(&body.id);
    if let Ok(id) = id {
        if let Ok(Some(doc)) = state.db.get_document(&DocumentId(id.try_into().unwrap())) {
            let what_if = doc.content;
            request.model = "gpt-4-turbo-preview".to_string();
            request.messages.push(ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: format!("```what if report\n\n{what_if}\n```\n\nGiven the above report. Create a schedule for the student. Make sure to take into account the classes they have currently taken and the required classes in the document above. Respond in valid json according to the following schema: {{user_message: \"message to the user explaining your reasoning for choosing each course / a general overview\", semesters: [{{year: \"year here\", classes: [{{code: \"title here\"}}]}}]}}. Do not change the title of classes or assume their names. Use the names listed in the document for the titles of the classes. In addition here is the student's major: {}, campus: {}, and an additional query from the user: \"{}\". That may be blank.", body.major, body.campus, body.query),
                role: Role::System,
                name: None,
            }));

            let resp = client.chat().create(request).await.unwrap();
            let content = resp.choices[0].message.content.clone();

            let resp: serde_json::Value = serde_json::from_str(&content.unwrap_or(String::from("{}"))).unwrap();
            return Ok(Json(resp));
            // println!("{}", content.unwrap_or_default());
            
        } else {
            return Err(JsonErrorResponse(
                StatusCode::BAD_REQUEST,
                format!("invalid document id"),
            ));
        }
    } else {
        return Err(JsonErrorResponse(
            StatusCode::BAD_REQUEST,
            format!("invalid hex id"),
        ));
    }

}

#[tokio::main]
async fn main() {
    let app_state = AppState::new("./database");

    let routes = Router::new()
        .route("/upload", post(upload))
        .route("/recommend", post(recommend))
        // .layer(CorsLayer::permissive())
        // .route("/majors", get(|| async {})) // all majors in the system
        // .route("/campus", get(|| async {})) // all campus that are valid
        .layer(Extension(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

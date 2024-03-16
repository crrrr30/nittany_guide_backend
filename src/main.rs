use axum::{extract::Multipart, http::StatusCode, routing::{get, post}, Extension, Router};
use state::AppState;

mod error;
mod db;
mod state;

async fn upload(state: Extension<AppState>, mut multipart: Multipart) -> StatusCode {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }

    StatusCode::OK
}


#[tokio::main]
async fn main() {

    let app_state = AppState::new("./database");

    let routes = Router::new()
        .route("/upload", post(upload))
        // .route("/majors", get(|| async {})) // all majors in the system
        // .route("/campus", get(|| async {})) // all campus that are valid
        .layer(Extension(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();

}

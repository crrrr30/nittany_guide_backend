use async_openai::{config::OpenAIConfig, Client};

use crate::db::Database;


/// Represents the shared state of your application.
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Database,
    pub client: Client<OpenAIConfig>
}

impl AppState {
    pub fn new(database_path: &str) -> Self {
        let db = Database::new(database_path).expect("failed to setup database");
        let client = Client::new();
        Self { db, client }
    }
}
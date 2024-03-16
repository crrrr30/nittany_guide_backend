use crate::db::Database;


/// Represents the shared state of your application.
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Database
}

impl AppState {
    pub fn new(database_path: &str) -> Self {
        let db = Database::new(database_path).expect("failed to setup database");
        Self { db }
    }
}
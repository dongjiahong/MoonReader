// Services module for business logic
use sqlx::SqlitePool;
use crate::database::DatabaseManager;

pub mod ai;

// Application state that will be shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseManager,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self { 
            db: DatabaseManager::new(pool)
        }
    }
}
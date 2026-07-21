use log::info;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::RwLock;

mod auth;
mod exercises;
pub mod idempotency;
pub mod mcp_tokens;
pub mod oauth;
mod progress;
mod progress_consistency;
mod templates;
mod workouts;

#[derive(Clone)]
pub struct Database {
    pool: Arc<RwLock<Pool<Sqlite>>>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        info!(target: "database", "Database connection pool initialized");
        Self {
            pool: Arc::new(RwLock::new(pool)),
        }
    }

    pub async fn pool(&self) -> Pool<Sqlite> {
        self.pool.read().await.clone()
    }

    pub async fn replace_pool(&self, new_pool: Pool<Sqlite>) {
        *self.pool.write().await = new_pool;
        info!(target: "database", "Database connection pool updated");
    }
}

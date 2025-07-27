use anyhow::Result;
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;

use crate::core::config::DatabaseConfig;

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", config.url);
        
        let pool = SqlitePool::connect(&config.url).await?;
        
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
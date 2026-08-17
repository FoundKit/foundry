use foundry_core::error::{AppError, AppResult};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

pub type DbPool = PgPool;

/// Initialize Postgres Connection Pool
pub async fn init_db_pool(database_url: &str, max_connections: u32) -> AppResult<DbPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect to PostgreSQL: {}", e)))
}

/// Run initial database migration script from `migrations/init.sql`
pub async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    info!("Running database initialization migrations...");
    let init_sql = include_str!("../../../migrations/init.sql");
    sqlx::raw_sql(init_sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;
    info!("Database migration completed successfully.");
    Ok(())
}

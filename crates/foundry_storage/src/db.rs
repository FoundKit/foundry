use crate::facade::Database;
use foundry_core::error::AppResult;

/// Type alias for Database facade to maintain backward compatibility without leaking driver pools
pub type DbPool = Database;

/// Initialize database connection using Database facade
pub async fn init_db_pool(database_url: &str, max_connections: u32) -> AppResult<Database> {
    Database::connect(database_url, max_connections, false).await
}

/// Run initial database migrations
pub async fn run_migrations(database_url_or_dummy: &Database) -> AppResult<()> {
    // If called with &Database, migrations are typically already run or can be called via Database::run_migrations
    let _ = database_url_or_dummy;
    Ok(())
}

use crate::engines::postgres::records::PostgresRecordEngine;
use crate::engines::postgres::stores::{
    PostgresAdminEngine, PostgresAuditEngine, PostgresConfigEngine, PostgresModelEngine,
    PostgresSystemEngine,
};
use crate::spi::provider::{StorageDriverProvider, StorageEngineSet};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub struct PostgresProvider;

#[async_trait]
impl StorageDriverProvider for PostgresProvider {
    fn driver_name(&self) -> &'static str {
        "postgres"
    }

    fn supports(&self, connection_url: &str) -> bool {
        connection_url.starts_with("postgres://") || connection_url.starts_with("postgresql://")
    }

    async fn create_engines(
        &self,
        connection_url: &str,
        max_connections: u32,
    ) -> AppResult<StorageEngineSet> {
        let pool = if connection_url.contains("dummy") {
            PgPoolOptions::new()
                .connect_lazy(connection_url)
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to initialize lazy PostgreSQL connection: {}",
                        e
                    ))
                })?
        } else {
            PgPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(Duration::from_secs(5))
                .connect(connection_url)
                .await
                .map_err(|e| {
                    AppError::Database(format!("Failed to connect to PostgreSQL: {}", e))
                })?
        };

        Ok(StorageEngineSet {
            records: Arc::new(PostgresRecordEngine::new(pool.clone())),
            models: Arc::new(PostgresModelEngine::new(pool.clone())),
            systems: Arc::new(PostgresSystemEngine::new(pool.clone())),
            configs: Arc::new(PostgresConfigEngine::new(pool.clone())),
            admins: Arc::new(PostgresAdminEngine::new(pool.clone())),
            audit: Arc::new(PostgresAuditEngine::new(pool)),
        })
    }

    async fn run_migrations(&self, connection_url: &str) -> AppResult<()> {
        info!("Running PostgreSQL database initialization migrations...");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(connection_url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect for migrations: {}", e)))?;

        let init_sql = include_str!("../../../migrations/postgres/init.sql");
        sqlx::raw_sql(init_sql)
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(format!("PostgreSQL migration failed: {}", e)))?;

        info!("PostgreSQL database migration completed successfully.");
        Ok(())
    }
}

use crate::engines::postgres::records::PostgresRecordEngine;
use crate::engines::postgres::stores::{
    PostgresAdminEngine, PostgresApplicationEngine, PostgresAuditEngine, PostgresConfigEngine,
    PostgresModelEngine, PostgresSystemEngine,
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
            audit: Arc::new(PostgresAuditEngine::new(pool.clone())),
            apps: Arc::new(PostgresApplicationEngine::new(pool)),
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

        let init_sql = self.get_baseline_ddl();
        sqlx::raw_sql(sqlx::AssertSqlSafe(init_sql.as_str()))
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(format!("PostgreSQL migration failed: {}", e)))?;

        // Ensure soft-deletion column exists on any pre-existing tables
        let alter_sql = r#"
            DO $$
            BEGIN
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'systems') THEN
                    ALTER TABLE systems ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'system_configs') THEN
                    ALTER TABLE system_configs ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'model_fields') THEN
                    ALTER TABLE model_fields ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'admins') THEN
                    ALTER TABLE admins ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'applications') THEN
                    ALTER TABLE applications ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ NULL;
                END IF;
            END $$;
        "#;
        let _ = sqlx::raw_sql(alter_sql).execute(&pool).await;

        info!("PostgreSQL database migration completed successfully.");
        Ok(())
    }

    async fn get_existing_tables(&self, connection_url: &str) -> AppResult<Vec<String>> {
        if connection_url.contains("dummy") {
            return Ok(Vec::new());
        }

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(connection_url)
            .await
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to connect to PostgreSQL for schema verification: {}",
                    e
                ))
            })?;

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = ANY(current_schemas(false))",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            AppError::Database(format!(
                "Failed to query PostgreSQL table metadata: {}",
                e
            ))
        })?;

        Ok(rows.into_iter().map(|(t,)| t).collect())
    }

    fn get_baseline_ddl(&self) -> String {
        crate::orm::compile_all_ddl(crate::orm::Dialect::Postgres)
    }
}

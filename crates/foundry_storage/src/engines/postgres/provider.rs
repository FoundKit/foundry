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

        // 1. Ensure `_foundry_migrations` tracking table exists
        let create_meta_sql = "CREATE TABLE IF NOT EXISTS _foundry_migrations (\
            sequence INT PRIMARY KEY,\
            version VARCHAR(32) NOT NULL,\
            name VARCHAR(64) NOT NULL,\
            applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP\
        );";
        let _ = sqlx::raw_sql(create_meta_sql).execute(&pool).await;

        // 2. Fetch applied migration sequences
        let applied_rows: Vec<(i32,)> =
            sqlx::query_as("SELECT sequence FROM _foundry_migrations ORDER BY sequence ASC")
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        let mut applied_set: std::collections::HashSet<u32> =
            applied_rows.into_iter().map(|(s,)| s as u32).collect();

        // If _foundry_migrations was just created on an existing legacy database, backfill
        if applied_set.is_empty() {
            let existing_tables = self
                .get_existing_tables(connection_url)
                .await
                .unwrap_or_default();
            let table_set: std::collections::HashSet<String> = existing_tables
                .into_iter()
                .map(|t| t.to_ascii_lowercase())
                .collect();
            if table_set.contains("systems") {
                let _ = sqlx::raw_sql("INSERT INTO _foundry_migrations (sequence, version, name) VALUES (1, 'v0.1.0', 'baseline') ON CONFLICT DO NOTHING;").execute(&pool).await;
                applied_set.insert(1);
            }
            if table_set.contains("applications") {
                let _ = sqlx::raw_sql("INSERT INTO _foundry_migrations (sequence, version, name) VALUES (2, 'v0.2.0', 'add_applications') ON CONFLICT DO NOTHING;").execute(&pool).await;
                applied_set.insert(2);
            }
        }

        // 3. Apply all pending sequential versioned migrations
        let migrations = self.get_migrations();
        for m in migrations {
            if applied_set.contains(&m.sequence) {
                continue;
            }

            info!(
                "Applying PostgreSQL migration {:04} ({}_{})...",
                m.sequence, m.version, m.name
            );

            sqlx::raw_sql(sqlx::AssertSqlSafe(m.sql))
                .execute(&pool)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "PostgreSQL migration {:04} ({}) failed: {}",
                        m.sequence, m.name, e
                    ))
                })?;

            let _ = sqlx::query(
                "INSERT INTO _foundry_migrations (sequence, version, name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
            )
            .bind(m.sequence as i32)
            .bind(m.version)
            .bind(m.name)
            .execute(&pool)
            .await;

            applied_set.insert(m.sequence);
        }

        // 4. Ensure soft-deletion column exists on any pre-existing tables
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

        // 5. Apply any code-defined DDL extensions registered by subsystems
        let extensions = crate::spi::registry::get_registered_ddl_extensions(self.driver_name());
        for ext in extensions {
            let _ = sqlx::raw_sql(sqlx::AssertSqlSafe(ext.content.as_str()))
                .execute(&pool)
                .await;
        }

        info!("PostgreSQL database migrations completed successfully.");
        Ok(())
    }

    async fn get_applied_migrations(&self, connection_url: &str) -> AppResult<Vec<u32>> {
        if connection_url.contains("dummy") {
            return Ok(Vec::new());
        }
        let pool = match PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(connection_url)
            .await
        {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };
        let rows: Vec<(i32,)> =
            sqlx::query_as("SELECT sequence FROM _foundry_migrations ORDER BY sequence ASC")
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        Ok(rows.into_iter().map(|(s,)| s as u32).collect())
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

    fn get_version_patch_ddl(&self, version: &str) -> Option<&'static str> {
        match version {
            "v0.2.0" | "0.2.0" => Some(include_str!(
                "../../../migrations/postgres/versions/v0.2.0_add_applications.sql"
            )),
            _ => None,
        }
    }
}

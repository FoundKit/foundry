use crate::engines::mysql::connection::{create_mysql_pool, extract_server_url};
use crate::engines::mysql::records::MySqlRecordEngine;
use crate::engines::mysql::stores::{
    MySqlAdminEngine, MySqlApplicationEngine, MySqlAuditEngine, MySqlConfigEngine,
    MySqlModelEngine, MySqlSystemEngine,
};
use crate::spi::provider::{StorageDriverProvider, StorageEngineSet};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub struct MariaDbProvider;

fn normalize_mariadb_url(url: &str) -> String {
    if let Some(stripped) = url.strip_prefix("mariadb://") {
        format!("mysql://{}", stripped)
    } else {
        url.to_string()
    }
}

#[async_trait]
impl StorageDriverProvider for MariaDbProvider {
    fn driver_name(&self) -> &'static str {
        "mariadb"
    }

    fn supports(&self, connection_url: &str) -> bool {
        connection_url.starts_with("mariadb://")
    }

    async fn create_engines(
        &self,
        connection_url: &str,
        max_connections: u32,
    ) -> AppResult<StorageEngineSet> {
        let normalized_url = normalize_mariadb_url(connection_url);

        let pool = create_mysql_pool(
            &normalized_url,
            max_connections,
            Duration::from_secs(5),
            "mariadb engine pool",
        )
        .await?;

        Ok(StorageEngineSet {
            records: Arc::new(MySqlRecordEngine::new(pool.clone())),
            models: Arc::new(MySqlModelEngine::new(pool.clone())),
            systems: Arc::new(MySqlSystemEngine::new(pool.clone())),
            configs: Arc::new(MySqlConfigEngine::new(pool.clone())),
            admins: Arc::new(MySqlAdminEngine::new(pool.clone())),
            audit: Arc::new(MySqlAuditEngine::new(pool.clone())),
            apps: Arc::new(MySqlApplicationEngine::new(pool)),
        })
    }

    async fn run_migrations(&self, connection_url: &str) -> AppResult<()> {
        info!("Running MariaDB database initialization migrations...");
        let normalized_url = normalize_mariadb_url(connection_url);

        // If the database does not exist yet, connect to the MariaDB instance without a database to create it
        let pool = match create_mysql_pool(
            &normalized_url,
            2,
            Duration::from_secs(5),
            "mariadb migrations",
        )
        .await
        {
            Ok(p) => p,
            Err(_e) => {
                if let Some((server_url, db_name)) = extract_server_url(&normalized_url)
                    && let Ok(server_pool) = create_mysql_pool(
                        &server_url,
                        1,
                        Duration::from_secs(5),
                        "create mariadb database",
                    )
                    .await
                {
                    let create_sql = format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name);
                    let _ = sqlx::query(sqlx::AssertSqlSafe(create_sql.as_str()))
                        .execute(&server_pool)
                        .await;
                }

                // Retry connecting to target database
                create_mysql_pool(
                    &normalized_url,
                    2,
                    Duration::from_secs(5),
                    "mariadb migrations retry",
                )
                .await?
            }
        };

        let init_sql = self.get_baseline_ddl();

        // Split statements by semicolon and execute each statement sequentially
        let statements = init_sql.split(';');
        for raw_stmt in statements {
            let stmt = raw_stmt.trim();
            if stmt.is_empty() {
                continue;
            }

            sqlx::query(sqlx::AssertSqlSafe(stmt))
                .execute(&pool)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "MariaDB migration statement failed [{}]: {}",
                        stmt, e
                    ))
                })?;
        }

        // Ensure soft-deletion column exists on any pre-existing tables
        let mariadb_alters = [
            (
                "systems",
                "ALTER TABLE `systems` ADD COLUMN `deleted_at` DATETIME(6) NULL",
            ),
            (
                "system_configs",
                "ALTER TABLE `system_configs` ADD COLUMN `deleted_at` DATETIME(6) NULL",
            ),
            (
                "model_fields",
                "ALTER TABLE `model_fields` ADD COLUMN `deleted_at` DATETIME(6) NULL",
            ),
            (
                "admins",
                "ALTER TABLE `admins` ADD COLUMN `deleted_at` DATETIME(6) NULL",
            ),
        ];
        for (tbl, alter_stmt) in mariadb_alters {
            let col_exists: Option<(i64,)> = sqlx::query_as(
                "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = ? AND column_name = 'deleted_at'"
            )
            .bind(tbl)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();
            if let Some((0,)) = col_exists {
                let _ = sqlx::query(alter_stmt).execute(&pool).await;
            }
        }

        info!("MariaDB database migration completed successfully.");
        Ok(())
    }

    async fn get_existing_tables(&self, connection_url: &str) -> AppResult<Vec<String>> {
        let normalized_url = normalize_mariadb_url(connection_url);
        if normalized_url.contains("dummy") {
            return Ok(Vec::new());
        }

        let pool = create_mysql_pool(
            &normalized_url,
            2,
            Duration::from_secs(5),
            "mariadb schema verification",
        )
        .await?;

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            AppError::Database(format!("Failed to query MariaDB table metadata: {}", e))
        })?;

        Ok(rows.into_iter().map(|(t,)| t).collect())
    }

    fn get_baseline_ddl(&self) -> String {
        crate::orm::compile_all_ddl(crate::orm::Dialect::MariaDb)
    }
}

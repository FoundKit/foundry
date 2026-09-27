use crate::engines::mysql::records::MySqlRecordEngine;
use crate::engines::mysql::stores::{
    MySqlAdminEngine, MySqlAuditEngine, MySqlConfigEngine, MySqlModelEngine, MySqlSystemEngine,
};
use crate::spi::provider::{StorageDriverProvider, StorageEngineSet};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use sqlx::mysql::MySqlPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub struct MySqlProvider;

#[async_trait]
impl StorageDriverProvider for MySqlProvider {
    fn driver_name(&self) -> &'static str {
        "mysql"
    }

    fn supports(&self, connection_url: &str) -> bool {
        connection_url.starts_with("mysql://") || connection_url.starts_with("mariadb://")
    }

    async fn create_engines(
        &self,
        connection_url: &str,
        max_connections: u32,
    ) -> AppResult<StorageEngineSet> {
        let pool = if connection_url.contains("dummy") {
            MySqlPoolOptions::new()
                .connect_lazy(connection_url)
                .map_err(|e| {
                    AppError::Database(format!("Failed to initialize lazy MySQL connection: {}", e))
                })?
        } else {
            MySqlPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(Duration::from_secs(5))
                .connect(connection_url)
                .await
                .map_err(|e| AppError::Database(format!("Failed to connect to MySQL: {}", e)))?
        };

        Ok(StorageEngineSet {
            records: Arc::new(MySqlRecordEngine::new(pool.clone())),
            models: Arc::new(MySqlModelEngine::new(pool.clone())),
            systems: Arc::new(MySqlSystemEngine::new(pool.clone())),
            configs: Arc::new(MySqlConfigEngine::new(pool.clone())),
            admins: Arc::new(MySqlAdminEngine::new(pool.clone())),
            audit: Arc::new(MySqlAuditEngine::new(pool)),
        })
    }

    async fn run_migrations(&self, connection_url: &str) -> AppResult<()> {
        info!("Running MySQL/MariaDB database initialization migrations...");

        // If the database does not exist yet, connect to the MySQL instance without a database to create it
        let pool = match MySqlPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(connection_url)
            .await
        {
            Ok(p) => p,
            Err(_e) => {
                // If database unknown error, try to extract db name from URL and create it
                if let Some(slash_idx) = connection_url.rfind('/') {
                    let (base, db_part) = connection_url.split_at(slash_idx);
                    let db_name = db_part
                        .trim_start_matches('/')
                        .split('?')
                        .next()
                        .unwrap_or("");
                    if !db_name.is_empty() {
                        let server_url = format!("{}/", base);
                        if let Ok(server_pool) = MySqlPoolOptions::new()
                            .max_connections(1)
                            .acquire_timeout(Duration::from_secs(5))
                            .connect(&server_url)
                            .await
                        {
                            let create_sql =
                                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name);
                            let _ = sqlx::query(&create_sql).execute(&server_pool).await;
                        }
                    }
                }

                // Retry connecting to target database
                MySqlPoolOptions::new()
                    .max_connections(2)
                    .acquire_timeout(Duration::from_secs(5))
                    .connect(connection_url)
                    .await
                    .map_err(|err| {
                        AppError::Database(format!(
                            "Failed to connect to MySQL for migrations: {}",
                            err
                        ))
                    })?
            }
        };

        let init_sql = include_str!("../../../migrations/mysql/init.sql");

        // Split statements by semicolon and execute each statement sequentially
        let statements = init_sql.split(';');
        for raw_stmt in statements {
            let stmt = raw_stmt.trim();
            if stmt.is_empty() {
                continue;
            }

            sqlx::query(stmt).execute(&pool).await.map_err(|e| {
                AppError::Database(format!(
                    "MySQL migration statement failed [{}]: {}",
                    stmt, e
                ))
            })?;
        }

        info!("MySQL/MariaDB database migration completed successfully.");
        Ok(())
    }
}

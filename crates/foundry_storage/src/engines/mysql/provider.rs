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

pub struct MySqlProvider;

#[async_trait]
impl StorageDriverProvider for MySqlProvider {
    fn driver_name(&self) -> &'static str {
        "mysql"
    }

    fn supports(&self, connection_url: &str) -> bool {
        #[cfg(feature = "mariadb")]
        {
            connection_url.starts_with("mysql://")
        }
        #[cfg(not(feature = "mariadb"))]
        {
            connection_url.starts_with("mysql://") || connection_url.starts_with("mariadb://")
        }
    }

    async fn create_engines(
        &self,
        connection_url: &str,
        max_connections: u32,
    ) -> AppResult<StorageEngineSet> {
        let pool = create_mysql_pool(
            connection_url,
            max_connections,
            Duration::from_secs(5),
            "engine pool",
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
        info!("Running MySQL/MariaDB database initialization migrations...");

        // If the database does not exist yet, connect to the MySQL instance without a database to create it
        let pool = match create_mysql_pool(connection_url, 2, Duration::from_secs(5), "migrations")
            .await
        {
            Ok(p) => p,
            Err(_e) => {
                // If database unknown error, try to extract db name from URL and create it
                if let Some((server_url, db_name)) = extract_server_url(connection_url)
                    && let Ok(server_pool) =
                        create_mysql_pool(&server_url, 1, Duration::from_secs(5), "create database")
                            .await
                {
                    let create_sql = format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name);
                    let _ = sqlx::query(sqlx::AssertSqlSafe(create_sql.as_str()))
                        .execute(&server_pool)
                        .await;
                }

                // Retry connecting to target database
                create_mysql_pool(
                    connection_url,
                    2,
                    Duration::from_secs(5),
                    "migrations retry",
                )
                .await?
            }
        };

        // 1. Ensure `_foundry_migrations` tracking table exists
        let create_meta_sql = "CREATE TABLE IF NOT EXISTS `_foundry_migrations` (\
            `sequence` INT NOT NULL PRIMARY KEY,\
            `version` VARCHAR(32) NOT NULL,\
            `name` VARCHAR(64) NOT NULL,\
            `applied_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)\
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;";
        let _ = sqlx::query(create_meta_sql).execute(&pool).await;

        // 2. Fetch applied migration sequences
        let applied_rows: Vec<(i32,)> =
            sqlx::query_as("SELECT sequence FROM `_foundry_migrations` ORDER BY sequence ASC")
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        let mut applied_set: std::collections::HashSet<u32> =
            applied_rows.into_iter().map(|(s,)| s as u32).collect();

        // If _foundry_migrations was just created on an existing legacy database, backfill already existing baseline tables
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
                let _ = sqlx::query("INSERT IGNORE INTO `_foundry_migrations` (`sequence`, `version`, `name`) VALUES (1, 'v0.1.0', 'baseline')").execute(&pool).await;
                applied_set.insert(1);
            }
            if table_set.contains("applications") {
                let _ = sqlx::query("INSERT IGNORE INTO `_foundry_migrations` (`sequence`, `version`, `name`) VALUES (2, 'v0.2.0', 'add_applications')").execute(&pool).await;
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
                "Applying MySQL migration {:04} ({}_{})...",
                m.sequence, m.version, m.name
            );

            let statements = m.sql.split(';');
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
                            "MySQL migration {:04} ({}) failed on statement [{}]: {}",
                            m.sequence, m.name, stmt, e
                        ))
                    })?;
            }

            let _ = sqlx::query(
                "INSERT INTO `_foundry_migrations` (`sequence`, `version`, `name`) VALUES (?, ?, ?)"
            )
            .bind(m.sequence as i32)
            .bind(m.version)
            .bind(m.name)
            .execute(&pool)
            .await;

            applied_set.insert(m.sequence);
        }

        // 4. Ensure soft-deletion column exists on any pre-existing tables
        let mysql_alters = [
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
        for (tbl, alter_stmt) in mysql_alters {
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

        // 5. Apply any code-defined DDL extensions registered by subsystems
        let extensions = crate::spi::registry::get_registered_ddl_extensions(self.driver_name());
        for ext in extensions {
            for raw_stmt in ext.content.split(';') {
                let stmt = raw_stmt.trim();
                if stmt.is_empty() {
                    continue;
                }
                let _ = sqlx::query(sqlx::AssertSqlSafe(stmt)).execute(&pool).await;
            }
        }

        info!("MySQL/MariaDB database migrations completed successfully.");
        Ok(())
    }

    async fn get_applied_migrations(&self, connection_url: &str) -> AppResult<Vec<u32>> {
        if connection_url.contains("dummy") {
            return Ok(Vec::new());
        }
        let pool = match create_mysql_pool(
            connection_url,
            1,
            Duration::from_secs(5),
            "applied migrations",
        )
        .await
        {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };
        let rows: Vec<(i32,)> =
            sqlx::query_as("SELECT sequence FROM `_foundry_migrations` ORDER BY sequence ASC")
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        Ok(rows.into_iter().map(|(s,)| s as u32).collect())
    }

    async fn get_existing_tables(&self, connection_url: &str) -> AppResult<Vec<String>> {
        if connection_url.contains("dummy") {
            return Ok(Vec::new());
        }

        let pool = create_mysql_pool(
            connection_url,
            2,
            Duration::from_secs(5),
            "schema verification",
        )
        .await?;

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE()",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query MySQL table metadata: {}", e)))?;

        Ok(rows.into_iter().map(|(t,)| t).collect())
    }

    fn get_baseline_ddl(&self) -> String {
        crate::orm::compile_all_ddl(crate::orm::Dialect::MySql)
    }

    fn get_version_patch_ddl(&self, version: &str) -> Option<&'static str> {
        match version {
            "v0.2.0" | "0.2.0" => Some(include_str!(
                "../../../migrations/mysql/versions/v0.2.0_add_applications.sql"
            )),
            _ => None,
        }
    }
}

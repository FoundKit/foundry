use crate::traits::{
    AdminStoreEngine, ApplicationStoreEngine, AuditStoreEngine, ConfigStoreEngine,
    ModelStoreEngine, RecordStoreEngine, SystemStoreEngine,
};
use async_trait::async_trait;
use foundry_core::error::AppResult;
use std::sync::Arc;

/// A collection of all storage engines instantiated for a specific driver
#[derive(Clone)]
pub struct StorageEngineSet {
    pub records: Arc<dyn RecordStoreEngine>,
    pub models: Arc<dyn ModelStoreEngine>,
    pub systems: Arc<dyn SystemStoreEngine>,
    pub configs: Arc<dyn ConfigStoreEngine>,
    pub admins: Arc<dyn AdminStoreEngine>,
    pub audit: Arc<dyn AuditStoreEngine>,
    pub apps: Arc<dyn ApplicationStoreEngine>,
}

/// A code-defined DDL extension providing custom schemas or table creation statements
#[derive(Debug, Clone)]
pub struct DdlExtension {
    /// Unique identifier / namespace of this extension, e.g. "subsystem:billing"
    pub identifier: String,
    /// Target database dialect: "postgres", "mysql", "mariadb", "mongodb", or "*" (all)
    pub db_type: String,
    /// The DDL SQL or initialization script content
    pub content: String,
}

#[async_trait]
pub trait StorageDriverProvider: Send + Sync {
    /// Unique driver identifier, e.g. "postgres", "mysql", "mariadb", "mongodb", "sqlite"
    fn driver_name(&self) -> &'static str;

    /// Checks if this provider supports the given connection URL (e.g. starts_with("mysql://"))
    fn supports(&self, connection_url: &str) -> bool;

    /// Initializes connection pool and constructs the complete set of store engines
    async fn create_engines(
        &self,
        connection_url: &str,
        max_connections: u32,
    ) -> AppResult<StorageEngineSet>;

    /// Executes database-specific initialization and migration scripts
    async fn run_migrations(&self, connection_url: &str) -> AppResult<()>;

    /// Retrieves all existing table (or collection) names currently in the database
    async fn get_existing_tables(&self, connection_url: &str) -> AppResult<Vec<String>>;

    /// Verifies whether the database contains all required tables/collections.
    /// If incomplete, returns an AppError::Database with diagnostic details.
    async fn verify_schema(&self, connection_url: &str, required_tables: &[&str]) -> AppResult<()> {
        if connection_url.contains("dummy") {
            return Ok(());
        }

        let existing = self.get_existing_tables(connection_url).await?;
        let existing_set: std::collections::HashSet<String> = existing
            .into_iter()
            .map(|t| t.to_ascii_lowercase())
            .collect();

        let mut missing = Vec::new();
        for req in required_tables {
            if !existing_set.contains(&req.to_ascii_lowercase()) {
                missing.push(*req);
            }
        }

        if !missing.is_empty() {
            let db_name = self.driver_name();
            let migration_file = self.default_migration_filename();
            return Err(foundry_core::error::AppError::Database(format!(
                "Database schema verification failed for driver '{}'!\n\
                The following required table(s) or collection(s) are missing from the database: {:?}.\n\
                Root cause: AUTO_MIGRATE=false is active, but the baseline database schema DDL has not been synchronized.\n\
                Remediation Options:\n\
                  1. Run platform migrations manually: `foundry migrate`\n\
                  2. Apply the dialect DDL script directly (e.g. `migrations/{}`)\n\
                  3. Or set AUTO_MIGRATE=true in your .env or environment to allow automatic schema synchronization.",
                db_name, missing, migration_file
            )));
        }

        Ok(())
    }

    /// Returns the baseline dialect DDL or initialization script compiled from ORM or embedded in the driver
    fn get_baseline_ddl(&self) -> String;

    /// Returns one-off incremental patch DDL for a given target version if applicable
    fn get_version_patch_ddl(&self, _version: &str) -> Option<&'static str> {
        None
    }

    /// Returns default filename for migrations (e.g. "init.sql" for SQL, "init.js" for MongoDB)
    fn default_migration_filename(&self) -> &'static str {
        "init.sql"
    }
}

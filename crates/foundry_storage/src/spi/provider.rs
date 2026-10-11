use crate::traits::{
    AdminStoreEngine, AuditStoreEngine, ConfigStoreEngine, ModelStoreEngine, RecordStoreEngine,
    SystemStoreEngine,
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

    /// Returns the baseline dialect DDL or initialization script compiled from ORM or embedded in the driver
    fn get_baseline_ddl(&self) -> String;

    /// Returns default filename for migrations (e.g. "init.sql" for SQL, "init.js" for MongoDB)
    fn default_migration_filename(&self) -> &'static str {
        "init.sql"
    }
}

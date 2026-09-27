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

#[async_trait]
pub trait StorageDriverProvider: Send + Sync {
    /// Unique driver identifier, e.g. "postgres", "mysql", "mariadb", "mongodb", "oracle", "sqlite"
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
}

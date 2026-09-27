use crate::spi::provider::StorageEngineSet;
use crate::spi::registry::StorageRegistry;
use crate::traits::{
    AdminStoreEngine, AuditStoreEngine, ConfigStoreEngine, ModelStoreEngine, RecordStoreEngine,
    SystemStoreEngine,
};
use foundry_core::error::AppResult;
use std::sync::Arc;

/// Unified database facade hiding all concrete database driver implementations
#[derive(Clone)]
pub struct Database {
    records: Arc<dyn RecordStoreEngine>,
    models: Arc<dyn ModelStoreEngine>,
    systems: Arc<dyn SystemStoreEngine>,
    configs: Arc<dyn ConfigStoreEngine>,
    admins: Arc<dyn AdminStoreEngine>,
    audit: Arc<dyn AuditStoreEngine>,
}

impl Database {
    /// Create Database facade directly from an engine set (useful for custom providers or mocks)
    pub fn from_engines(engines: StorageEngineSet) -> Self {
        Self {
            records: engines.records,
            models: engines.models,
            systems: engines.systems,
            configs: engines.configs,
            admins: engines.admins,
            audit: engines.audit,
        }
    }

    /// Creates an offline dummy Database instance for route and controller tests
    #[cfg(feature = "postgres")]
    pub fn dummy() -> Self {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://dummy:dummy@localhost:5432/dummy")
            .expect("Failed to create dummy pool");
        Self {
            records: Arc::new(
                crate::engines::postgres::records::PostgresRecordEngine::new(pool.clone()),
            ),
            models: Arc::new(crate::engines::postgres::stores::PostgresModelEngine::new(
                pool.clone(),
            )),
            systems: Arc::new(crate::engines::postgres::stores::PostgresSystemEngine::new(
                pool.clone(),
            )),
            configs: Arc::new(crate::engines::postgres::stores::PostgresConfigEngine::new(
                pool.clone(),
            )),
            admins: Arc::new(crate::engines::postgres::stores::PostgresAdminEngine::new(
                pool.clone(),
            )),
            audit: Arc::new(crate::engines::postgres::stores::PostgresAuditEngine::new(
                pool,
            )),
        }
    }

    /// Automatically routes by protocol or explicit DATABASE_TYPE env var
    pub async fn connect(url: &str, max_connections: u32, auto_migrate: bool) -> AppResult<Self> {
        let explicit_type = std::env::var("DATABASE_TYPE").ok();
        Self::connect_with_type(url, explicit_type.as_deref(), max_connections, auto_migrate).await
    }

    /// Connect with optional explicit database type ("postgres", "mysql", etc.)
    pub async fn connect_with_type(
        url: &str,
        explicit_type: Option<&str>,
        max_connections: u32,
        auto_migrate: bool,
    ) -> AppResult<Self> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider(url, explicit_type)?;

        if auto_migrate {
            provider.run_migrations(url).await?;
        }

        let engines = provider.create_engines(url, max_connections).await?;
        Ok(Self::from_engines(engines))
    }

    /// Run migrations for the specified database connection
    pub async fn run_migrations(url: &str, explicit_type: Option<&str>) -> AppResult<()> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider(url, explicit_type)?;
        provider.run_migrations(url).await
    }

    pub fn records(&self) -> &dyn RecordStoreEngine {
        self.records.as_ref()
    }

    pub fn models(&self) -> &dyn ModelStoreEngine {
        self.models.as_ref()
    }

    pub fn systems(&self) -> &dyn SystemStoreEngine {
        self.systems.as_ref()
    }

    pub fn configs(&self) -> &dyn ConfigStoreEngine {
        self.configs.as_ref()
    }

    /// Strongly-typed retrieval of sub-system configuration into a Rust struct
    pub async fn get_typed_config<T: serde::de::DeserializeOwned>(
        &self,
        system_slug: &str,
    ) -> AppResult<T> {
        let agg = self.configs().get_aggregated(system_slug).await?;
        serde_json::from_value::<T>(agg).map_err(|e| {
            foundry_core::error::AppError::Validation(format!(
                "Failed to deserialize typed config: {}",
                e
            ))
        })
    }

    pub fn admins(&self) -> &dyn AdminStoreEngine {
        self.admins.as_ref()
    }

    pub fn audit(&self) -> &dyn AuditStoreEngine {
        self.audit.as_ref()
    }
}

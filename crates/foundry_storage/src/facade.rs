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

    /// Creates an offline dummy Database instance for route and controller tests when running with MySQL only
    #[cfg(all(feature = "mysql", not(feature = "postgres")))]
    pub fn dummy() -> Self {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .connect_lazy("mysql://dummy:dummy@localhost:3306/dummy")
            .expect("Failed to create dummy pool");
        Self {
            records: Arc::new(crate::engines::mysql::records::MySqlRecordEngine::new(
                pool.clone(),
            )),
            models: Arc::new(crate::engines::mysql::stores::MySqlModelEngine::new(
                pool.clone(),
            )),
            systems: Arc::new(crate::engines::mysql::stores::MySqlSystemEngine::new(
                pool.clone(),
            )),
            configs: Arc::new(crate::engines::mysql::stores::MySqlConfigEngine::new(
                pool.clone(),
            )),
            admins: Arc::new(crate::engines::mysql::stores::MySqlAdminEngine::new(
                pool.clone(),
            )),
            audit: Arc::new(crate::engines::mysql::stores::MySqlAuditEngine::new(pool)),
        }
    }

    /// Creates an offline dummy Database instance for route and controller tests when running with MariaDB only
    #[cfg(all(feature = "mariadb", not(feature = "postgres"), not(feature = "mysql")))]
    pub fn dummy() -> Self {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .connect_lazy("mysql://dummy:dummy@localhost:3307/dummy")
            .expect("Failed to create dummy pool");
        Self {
            records: Arc::new(crate::engines::mysql::records::MySqlRecordEngine::new(
                pool.clone(),
            )),
            models: Arc::new(crate::engines::mysql::stores::MySqlModelEngine::new(
                pool.clone(),
            )),
            systems: Arc::new(crate::engines::mysql::stores::MySqlSystemEngine::new(
                pool.clone(),
            )),
            configs: Arc::new(crate::engines::mysql::stores::MySqlConfigEngine::new(
                pool.clone(),
            )),
            admins: Arc::new(crate::engines::mysql::stores::MySqlAdminEngine::new(
                pool.clone(),
            )),
            audit: Arc::new(crate::engines::mysql::stores::MySqlAuditEngine::new(pool)),
        }
    }

    /// Creates an offline dummy Database instance for route and controller tests when running with MongoDB only
    #[cfg(all(
        feature = "mongodb",
        not(feature = "postgres"),
        not(feature = "mysql"),
        not(feature = "mariadb")
    ))]
    pub fn dummy() -> Self {
        use std::future::IntoFuture;
        let client_options = match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| {
                handle.block_on(
                    mongodb::options::ClientOptions::parse("mongodb://127.0.0.1:27017")
                        .into_future(),
                )
            }),
            Err(_) => tokio::runtime::Runtime::new().unwrap().block_on(
                mongodb::options::ClientOptions::parse("mongodb://127.0.0.1:27017").into_future(),
            ),
        }
        .expect("Failed to parse dummy MongoDB options");
        let client = mongodb::Client::with_options(client_options)
            .expect("Failed to create dummy MongoDB client");
        let db = client.database("dummy");
        Self {
            records: Arc::new(crate::engines::mongodb::records::MongoRecordEngine::new(
                db.clone(),
            )),
            models: Arc::new(crate::engines::mongodb::stores::MongoModelEngine::new(
                db.clone(),
            )),
            systems: Arc::new(crate::engines::mongodb::stores::MongoSystemEngine::new(
                db.clone(),
            )),
            configs: Arc::new(crate::engines::mongodb::stores::MongoConfigEngine::new(
                db.clone(),
            )),
            admins: Arc::new(crate::engines::mongodb::stores::MongoAdminEngine::new(
                db.clone(),
            )),
            audit: Arc::new(crate::engines::mongodb::stores::MongoAuditEngine::new(db)),
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
        Self::connect_with_validation_control(
            url,
            explicit_type,
            max_connections,
            auto_migrate,
            false,
        )
        .await
    }

    /// Connect with explicit database type and validation control
    pub async fn connect_with_validation_control(
        url: &str,
        explicit_type: Option<&str>,
        max_connections: u32,
        auto_migrate: bool,
        skip_schema_validation: bool,
    ) -> AppResult<Self> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider(url, explicit_type)?;

        if auto_migrate {
            provider.run_migrations(url).await?;
        } else if !url.contains("dummy") && !skip_schema_validation {
            let env_skip = std::env::var("SKIP_SCHEMA_VALIDATION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            if !env_skip {
                let models = crate::orm::get_all_orm_models();
                let table_names: Vec<&str> = models.iter().map(|m| m.table_name.as_str()).collect();
                provider.verify_schema(url, &table_names).await?;
            }
        }

        let engines = provider.create_engines(url, max_connections).await?;
        Ok(Self::from_engines(engines))
    }

    /// Verifies that the database schema is complete and contains all required tables/collections.
    pub async fn verify_schema(url: &str, explicit_type: Option<&str>) -> AppResult<()> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider(url, explicit_type)?;
        let models = crate::orm::get_all_orm_models();
        let table_names: Vec<&str> = models.iter().map(|m| m.table_name.as_str()).collect();
        provider.verify_schema(url, &table_names).await
    }

    /// Returns the complete dialect DDL or initialization script generated directly from the binary SPI,
    /// including any code-defined DDL extensions registered by subsystems, plugins, or application code.
    pub fn get_ddl(explicit_type: &str) -> AppResult<String> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider_by_type(explicit_type)?;
        let baseline = provider.get_baseline_ddl();
        let extensions =
            crate::spi::registry::get_registered_ddl_extensions(provider.driver_name());

        if extensions.is_empty() {
            return Ok(baseline);
        }

        let mut full_ddl = baseline;
        full_ddl.push_str(
            "\n\n-- ============================================================================\n",
        );
        full_ddl.push_str("-- CODE-DEFINED EXTENSION DDL (Custom Subsystems & Extensions)\n");
        full_ddl.push_str(
            "-- ============================================================================\n\n",
        );

        for ext in extensions {
            full_ddl.push_str(&format!(
                "-- [Extension: {}] (Dialect: {})\n",
                ext.identifier, ext.db_type
            ));
            full_ddl.push_str(&ext.content);
            if !ext.content.ends_with('\n') {
                full_ddl.push('\n');
            }
            full_ddl.push('\n');
        }

        Ok(full_ddl)
    }

    /// Returns the default migration script filename ("init.sql" or "init.js")
    pub fn get_default_migration_filename(explicit_type: &str) -> AppResult<&'static str> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider_by_type(explicit_type)?;
        Ok(provider.default_migration_filename())
    }

    /// Run migrations for the specified database connection, including any code-defined DDL extensions
    pub async fn run_migrations(url: &str, explicit_type: Option<&str>) -> AppResult<()> {
        let registry = StorageRegistry::default_registry();
        let provider = registry.find_provider(url, explicit_type)?;
        provider.run_migrations(url).await?;

        // Apply registered code-defined DDL extensions for this dialect if present
        let extensions =
            crate::spi::registry::get_registered_ddl_extensions(provider.driver_name());
        if !extensions.is_empty() {
            crate::spi::registry::apply_code_extensions_migrations(
                provider.driver_name(),
                url,
                &extensions,
            )
            .await?;
        }

        Ok(())
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

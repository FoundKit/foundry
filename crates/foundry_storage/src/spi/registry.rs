use crate::spi::provider::{DdlExtension, StorageDriverProvider};
use foundry_core::error::{AppError, AppResult};
use std::sync::{Arc, RwLock};

static CUSTOM_PROVIDERS: RwLock<Vec<Arc<dyn StorageDriverProvider>>> = RwLock::new(Vec::new());
static DDL_EXTENSIONS: RwLock<Vec<DdlExtension>> = RwLock::new(Vec::new());

/// Register a custom storage driver provider into the global SPI registry via code
pub fn register_storage_provider(provider: Arc<dyn StorageDriverProvider>) {
    let mut providers = CUSTOM_PROVIDERS
        .write()
        .expect("CUSTOM_PROVIDERS lock poisoned");
    providers.push(provider);
}

/// Register a code-defined DDL extension snippet into the binary storage runtime
pub fn register_ddl_extension(
    identifier: impl Into<String>,
    db_type: impl Into<String>,
    content: impl Into<String>,
) {
    let mut exts = DDL_EXTENSIONS
        .write()
        .expect("DDL_EXTENSIONS lock poisoned");
    exts.push(DdlExtension {
        identifier: identifier.into(),
        db_type: db_type.into(),
        content: content.into(),
    });
}

/// Retrieve all registered DDL extensions for a target database engine
pub fn get_registered_ddl_extensions(target_db: &str) -> Vec<DdlExtension> {
    let exts = DDL_EXTENSIONS.read().expect("DDL_EXTENSIONS lock poisoned");
    exts.iter()
        .filter(|e| e.db_type == "*" || e.db_type.eq_ignore_ascii_case(target_db))
        .cloned()
        .collect()
}

/// Clears registered DDL extensions (useful for unit tests)
pub fn clear_registered_ddl_extensions() {
    let mut exts = DDL_EXTENSIONS
        .write()
        .expect("DDL_EXTENSIONS lock poisoned");
    exts.clear();
}

pub struct StorageRegistry {
    providers: Vec<Arc<dyn StorageDriverProvider>>,
}

impl Default for StorageRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

impl StorageRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register(&mut self, provider: Arc<dyn StorageDriverProvider>) {
        self.providers.push(provider);
    }

    pub fn find_provider(
        &self,
        url: &str,
        explicit_type: Option<&str>,
    ) -> AppResult<&dyn StorageDriverProvider> {
        if let Some(dtype) = explicit_type {
            return self.find_provider_by_type(dtype);
        }

        for p in &self.providers {
            if p.supports(url) {
                return Ok(p.as_ref());
            }
        }

        Err(AppError::Internal(format!(
            "No storage driver provider supports connection URL '{}'. Please check your DATABASE_URL format or specify DATABASE_TYPE. Registered drivers: {:?}",
            url,
            self.providers
                .iter()
                .map(|p| p.driver_name())
                .collect::<Vec<_>>()
        )))
    }

    pub fn find_provider_by_type(&self, db_type: &str) -> AppResult<&dyn StorageDriverProvider> {
        let lower = db_type.trim().to_lowercase();
        let target = match lower.as_str() {
            "postgresql" | "pg" => "postgres",
            "mongo" => "mongodb",
            other => other,
        };
        if let Some(p) = self
            .providers
            .iter()
            .find(|p| p.driver_name().eq_ignore_ascii_case(target))
        {
            return Ok(p.as_ref());
        }
        Err(AppError::Internal(format!(
            "No storage driver provider matches database type '{}'. Enabled drivers: {:?}",
            db_type,
            self.providers
                .iter()
                .map(|p| p.driver_name())
                .collect::<Vec<_>>()
        )))
    }

    /// Creates a default registry populated with all compiled-in providers based on Cargo features,
    /// along with any dynamically registered code-defined custom providers.
    pub fn default_registry() -> Self {
        let mut registry = Self::new();

        #[cfg(feature = "postgres")]
        {
            registry.register(Arc::new(crate::engines::postgres::PostgresProvider));
        }

        #[cfg(feature = "mysql")]
        {
            registry.register(Arc::new(crate::engines::mysql::MySqlProvider));
        }

        #[cfg(feature = "mariadb")]
        {
            registry.register(Arc::new(crate::engines::mariadb::MariaDbProvider));
        }

        #[cfg(feature = "mongodb")]
        {
            registry.register(Arc::new(crate::engines::mongodb::MongoProvider));
        }

        // Add code-defined custom providers registered via register_storage_provider()
        if let Ok(custom) = CUSTOM_PROVIDERS.read() {
            for p in custom.iter() {
                registry.register(Arc::clone(p));
            }
        }

        registry
    }
}

/// Applies registered code-defined DDL extensions to the target database
pub async fn apply_code_extensions_migrations(
    driver_name: &str,
    url: &str,
    extensions: &[DdlExtension],
) -> AppResult<()> {
    let _ = url;
    match driver_name {
        #[cfg(feature = "postgres")]
        "postgres" => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(url)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to connect to PostgreSQL for extension migrations: {}",
                        e
                    ))
                })?;
            for ext in extensions {
                tracing::info!(
                    "Applying code-defined DDL extension '{}' for PostgreSQL...",
                    ext.identifier
                );
                sqlx::raw_sql(sqlx::AssertSqlSafe(ext.content.as_str()))
                    .execute(&pool)
                    .await
                    .map_err(|e| {
                        AppError::Database(format!(
                            "Failed to execute code-defined DDL extension '{}': {}",
                            ext.identifier, e
                        ))
                    })?;
            }
        }
        #[cfg(any(feature = "mysql", feature = "mariadb"))]
        "mysql" | "mariadb" => {
            let normalized_url = if driver_name == "mariadb" {
                url.replace("mariadb://", "mysql://")
            } else {
                url.to_string()
            };
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(2)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(&normalized_url)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to connect to MySQL/MariaDB for extension migrations: {}",
                        e
                    ))
                })?;
            for ext in extensions {
                tracing::info!(
                    "Applying code-defined DDL extension '{}' for MySQL/MariaDB...",
                    ext.identifier
                );
                for stmt in ext.content.split(';') {
                    let s = stmt.trim();
                    if !s.is_empty() {
                        sqlx::query(sqlx::AssertSqlSafe(s))
                            .execute(&pool)
                            .await
                            .map_err(|e| {
                                AppError::Database(format!(
                                    "Failed to execute extension statement in '{}' [{}]: {}",
                                    ext.identifier, s, e
                                ))
                            })?;
                    }
                }
            }
        }
        #[cfg(feature = "mongodb")]
        "mongodb" => {
            for ext in extensions {
                tracing::info!(
                    "Noted code-defined extension '{}' for MongoDB (registered in binary)",
                    ext.identifier
                );
            }
        }
        _ => {
            tracing::debug!(
                "No specialized extension runner for driver '{}', skipped",
                driver_name
            );
        }
    }
    Ok(())
}

use crate::spi::provider::StorageDriverProvider;
use foundry_core::error::{AppError, AppResult};

pub struct StorageRegistry {
    providers: Vec<Box<dyn StorageDriverProvider>>,
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

    pub fn register(&mut self, provider: Box<dyn StorageDriverProvider>) {
        self.providers.push(provider);
    }

    pub fn find_provider(
        &self,
        url: &str,
        explicit_type: Option<&str>,
    ) -> AppResult<&dyn StorageDriverProvider> {
        if let Some(dtype) = explicit_type {
            let lower = dtype.trim().to_lowercase();
            if let Some(p) = self
                .providers
                .iter()
                .find(|p| p.driver_name().eq_ignore_ascii_case(&lower))
            {
                return Ok(p.as_ref());
            }
            return Err(AppError::Internal(format!(
                "Configured DATABASE_TYPE='{}' but no matching storage driver provider was registered. Enabled drivers: {:?}",
                dtype,
                self.providers
                    .iter()
                    .map(|p| p.driver_name())
                    .collect::<Vec<_>>()
            )));
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

    /// Creates a default registry populated with all compiled-in providers based on Cargo features
    pub fn default_registry() -> Self {
        let mut registry = Self::new();

        #[cfg(feature = "postgres")]
        {
            registry.register(Box::new(crate::engines::postgres::PostgresProvider));
        }

        #[cfg(feature = "mysql")]
        {
            registry.register(Box::new(crate::engines::mysql::MySqlProvider));
        }

        registry
    }
}

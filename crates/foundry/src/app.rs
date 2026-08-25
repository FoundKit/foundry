use axum::Router;
use foundry_auth::JwtService;
use foundry_core::SubsystemModule;
use foundry_core::error::{AppError, AppResult};
use foundry_engine::{AppState, build_router, load_external_subsystems};
use foundry_extension::{HookPipeline, MutationHook};
use foundry_storage::{DbPool, RedisPool, init_db_pool, init_redis, run_migrations};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Runtime configuration for a Foundry application
#[derive(Debug, Clone)]
pub struct FoundryConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub auto_migrate: bool,
    pub db_pool_size: u32,
    pub external_systems_dirs: Vec<PathBuf>,
    pub admin_static_dirs: Vec<PathBuf>,
}

impl Default for FoundryConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
            }),
            redis_url: std::env::var("REDIS_URL").ok(),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                "foundry_super_secret_jwt_key_2026_change_in_production".to_string()
            }),
            jwt_expiry_hours: 24 * 7,
            auto_migrate: std::env::var("AUTO_MIGRATE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            db_pool_size: 20,
            external_systems_dirs: Vec::new(),
            admin_static_dirs: Vec::new(),
        }
    }
}

impl FoundryConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        Self::default()
    }
}

/// Fluent builder for constructing and starting a Foundry Application
pub struct FoundryBuilder {
    config: FoundryConfig,
    subsystems: Vec<Box<dyn SubsystemModule>>,
    hooks: HookPipeline,
    init_tracing: bool,
}

impl Default for FoundryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FoundryBuilder {
    pub fn new() -> Self {
        let _ = dotenvy::dotenv();
        Self {
            config: FoundryConfig::default(),
            subsystems: Vec::new(),
            hooks: HookPipeline::new(),
            init_tracing: true,
        }
    }

    pub fn config(mut self, config: FoundryConfig) -> Self {
        self.config = config;
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    pub fn redis_url(mut self, url: Option<String>) -> Self {
        self.config.redis_url = url;
        self
    }

    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.jwt_secret = secret.into();
        self
    }

    pub fn jwt_expiry_hours(mut self, hours: i64) -> Self {
        self.config.jwt_expiry_hours = hours;
        self
    }

    pub fn auto_migrate(mut self, auto: bool) -> Self {
        self.config.auto_migrate = auto;
        self
    }

    pub fn db_pool_size(mut self, size: u32) -> Self {
        self.config.db_pool_size = size;
        self
    }

    pub fn init_tracing(mut self, init: bool) -> Self {
        self.init_tracing = init;
        self
    }

    pub fn register_subsystem<S: SubsystemModule>(mut self, subsystem: S) -> Self {
        self.subsystems.push(Box::new(subsystem));
        self
    }

    pub fn register_boxed_subsystem(mut self, subsystem: Box<dyn SubsystemModule>) -> Self {
        self.subsystems.push(subsystem);
        self
    }

    pub fn register_hook<H: MutationHook>(mut self, hook: H) -> Self {
        self.hooks.register(hook);
        self
    }

    pub fn with_external_systems_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.external_systems_dirs.push(path.into());
        self
    }

    pub fn with_admin_spa_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.admin_static_dirs.push(path.into());
        self
    }

    /// Build the application runtime and router
    pub async fn build(self) -> AppResult<FoundryApp> {
        if self.init_tracing {
            let _ = tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info,foundry=debug,foundry_engine=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .try_init();
        }

        info!("Initializing Foundry Platform Application...");

        // 1. Initialize PostgreSQL Connection Pool
        info!("Connecting to PostgreSQL database...");
        let db_pool = init_db_pool(&self.config.database_url, self.config.db_pool_size).await?;

        // 2. Run migrations if enabled
        if self.config.auto_migrate {
            run_migrations(&db_pool).await?;
        }

        // 3. Initialize Redis if configured
        let redis_pool = if let Some(ref rurl) = self.config.redis_url {
            match init_redis(rurl).await {
                Ok(pool) => Some(pool),
                Err(e) => {
                    tracing::warn!(
                        "Redis connection skipped or failed: {}. Continuing without Redis cache.",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        // 4. Setup Auth
        let jwt_service =
            JwtService::new(self.config.jwt_secret.clone(), self.config.jwt_expiry_hours);

        // 5. Gather Subsystems (both code-first and external manifest)
        let mut all_subsystems = self.subsystems;
        let external_subsystems = load_external_subsystems(&self.config.external_systems_dirs);
        all_subsystems.extend(external_subsystems);

        info!("Loaded {} active sub-systems", all_subsystems.len());

        let state = AppState::new(db_pool, redis_pool, jwt_service, self.hooks, all_subsystems);

        let router = build_router(state.clone());

        Ok(FoundryApp {
            config: self.config,
            state,
            router,
        })
    }
}

/// The instantiated Foundry Application instance ready to serve traffic
pub struct FoundryApp {
    pub config: FoundryConfig,
    pub state: AppState,
    pub router: Router,
}

impl FoundryApp {
    pub fn builder() -> FoundryBuilder {
        FoundryBuilder::new()
    }

    pub fn router(&self) -> Router {
        self.router.clone()
    }

    pub fn into_router(self) -> Router {
        self.router
    }

    pub fn db_pool(&self) -> &DbPool {
        &self.state.db
    }

    pub fn redis_pool(&self) -> Option<&RedisPool> {
        self.state.redis.as_ref()
    }

    /// Run the application using the configured host and port
    pub async fn run(self) -> anyhow::Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid listen address: {}", e)))?;
        self.serve(addr).await
    }

    /// Run the application on a specific socket address
    pub async fn serve(self, addr: SocketAddr) -> anyhow::Result<()> {
        info!("🚀 Foundry Application listening on http://{}", addr);
        info!("📚 REST APIs available at http://{}/api/v1", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

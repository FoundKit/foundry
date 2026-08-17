use foundry_auth::JwtService;
use foundry_engine::{AppState, build_router};
use foundry_extension::HookPipeline;
use foundry_storage::{init_db_pool, init_redis, run_migrations};
use std::env;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load .env file
    let _ = dotenvy::dotenv();

    // 2. Initialize tracing logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,foundry_server=debug,foundry_engine=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Foundry Platform Engine...");

    // 3. Read configuration
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
    });
    let redis_url = env::var("REDIS_URL").ok();
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "foundry_super_secret_jwt_key_2026_change_in_production".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    // 4. Initialize Database
    info!("Connecting to PostgreSQL database...");
    let db_pool = init_db_pool(&database_url, 20).await?;

    // Run migrations if AUTO_MIGRATE=true (default: true)
    let auto_migrate = env::var("AUTO_MIGRATE").unwrap_or_else(|_| "true".to_string()) == "true";
    if auto_migrate {
        run_migrations(&db_pool).await?;
    }

    // 5. Initialize Redis (optional)
    let redis_pool = if let Some(ref rurl) = redis_url {
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

    // 6. Setup Auth and Extension Pipeline
    let jwt_service = JwtService::new(jwt_secret, 24 * 7); // 7 days token expiry
    let hook_pipeline = HookPipeline::new();

    let state = AppState::new(db_pool, redis_pool, jwt_service, hook_pipeline);

    // 7. Load Subsystems from `systems/` registry
    let subsystems = systems::register_subsystems();
    info!("Loaded {} custom sub-systems", subsystems.len());

    // 8. Build Router
    let app = build_router(state, subsystems);

    // 9. Bind and Serve
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("🚀 Foundry Server listening on http://{}", addr);
    info!("📚 REST APIs available at http://{}/api/v1", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

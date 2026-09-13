use foundry_auth::JwtService;
use foundry_core::SubsystemModule;
use foundry_extension::HookPipeline;
use foundry_storage::{DbPool, RedisPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: Option<RedisPool>,
    pub jwt: Arc<JwtService>,
    pub hooks: Arc<HookPipeline>,
    pub subsystems: Arc<Vec<Box<dyn SubsystemModule>>>,
}

impl AppState {
    pub fn new(
        db: DbPool,
        redis: Option<RedisPool>,
        jwt: JwtService,
        hooks: HookPipeline,
        subsystems: Vec<Box<dyn SubsystemModule>>,
    ) -> Self {
        Self {
            db,
            redis,
            jwt: Arc::new(jwt),
            hooks: Arc::new(hooks),
            subsystems: Arc::new(subsystems),
        }
    }
}

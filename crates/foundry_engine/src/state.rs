use crate::services::VerificationService;
use foundry_auth::JwtService;
use foundry_core::SubsystemModule;
use foundry_extension::HookPipeline;
use foundry_storage::{Database, RedisPool};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: Option<RedisPool>,
    pub jwt: Arc<JwtService>,
    pub hooks: Arc<HookPipeline>,
    pub subsystems: Arc<Vec<Box<dyn SubsystemModule>>>,
    pub verification: Arc<VerificationService>,
}

impl AppState {
    pub fn new(
        db: Database,
        redis: Option<RedisPool>,
        jwt: JwtService,
        hooks: HookPipeline,
        subsystems: Vec<Box<dyn SubsystemModule>>,
    ) -> Self {
        let verification = Arc::new(VerificationService::new(redis.clone()));
        Self {
            db,
            redis,
            jwt: Arc::new(jwt),
            hooks: Arc::new(hooks),
            subsystems: Arc::new(subsystems),
            verification,
        }
    }
}

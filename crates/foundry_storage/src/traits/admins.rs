use crate::entities::AdminEntity;
use async_trait::async_trait;
use foundry_core::error::AppResult;
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
pub trait AdminStoreEngine: Send + Sync {
    async fn list(&self) -> AppResult<Vec<AdminEntity>>;

    async fn get_by_id(&self, id: Uuid) -> AppResult<AdminEntity>;

    async fn get_by_username(&self, username: &str) -> AppResult<AdminEntity>;

    async fn create(
        &self,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: &str,
        allowed_systems: Value,
    ) -> AppResult<AdminEntity>;

    async fn update(
        &self,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<Value>,
        status: Option<i16>,
    ) -> AppResult<AdminEntity>;
}

use crate::entities::AuditLogEntity;
use async_trait::async_trait;
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogInsert {
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub system_slug: Option<String>,
    pub method: String,
    pub path: String,
    pub action_name: Option<String>,
    pub headers: serde_json::Value,
    pub query_params: Option<String>,
    pub body_params: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status_code: Option<i16>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuditLogQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub admin_id: Option<Uuid>,
    pub system_slug: Option<String>,
    pub method: Option<String>,
}

#[async_trait]
pub trait AuditStoreEngine: Send + Sync {
    async fn insert(&self, log: AuditLogInsert) -> AppResult<()>;

    async fn list(&self, query: AuditLogQuery) -> AppResult<PaginatedData<AuditLogEntity>>;
}

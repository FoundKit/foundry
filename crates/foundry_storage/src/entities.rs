use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemEntity {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfigEntity {
    pub id: i64,
    pub system_id: String,
    pub key: String,
    pub label: String,
    pub value_type: String,
    pub value: Option<serde_json::Value>,
    pub options: serde_json::Value,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelEntity {
    pub id: i64,
    pub system_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub status: i16,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelFieldEntity {
    pub id: i64,
    pub model_id: i64,
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub is_required: bool,
    pub default_value: Option<serde_json::Value>,
    pub options: serde_json::Value,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelRecordEntity {
    pub id: i64,
    pub system_id: String,
    pub model_slug: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminEntity {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub allowed_systems: serde_json::Value,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogEntity {
    pub id: i64,
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
    pub created_at: DateTime<Utc>,
}

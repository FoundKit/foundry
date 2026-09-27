use crate::entities::AdminEntity;
use crate::facade::Database;
use foundry_core::error::AppResult;
use uuid::Uuid;

pub struct AdminStore;

impl AdminStore {
    pub async fn list(db: &Database) -> AppResult<Vec<AdminEntity>> {
        db.admins().list().await
    }

    pub async fn get_by_id(db: &Database, id: Uuid) -> AppResult<AdminEntity> {
        db.admins().get_by_id(id).await
    }

    pub async fn get_by_username(db: &Database, username: &str) -> AppResult<AdminEntity> {
        db.admins().get_by_username(username).await
    }

    pub async fn create(
        db: &Database,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: &str,
        allowed_systems: serde_json::Value,
    ) -> AppResult<AdminEntity> {
        db.admins()
            .create(username, email, password_hash, role, allowed_systems)
            .await
    }

    pub async fn update(
        db: &Database,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<serde_json::Value>,
        status: Option<i16>,
    ) -> AppResult<AdminEntity> {
        db.admins()
            .update(id, email, password_hash, role, allowed_systems, status)
            .await
    }
}

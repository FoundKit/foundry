use crate::entities::ApplicationEntity;
use async_trait::async_trait;
use foundry_core::error::AppResult;
use uuid::Uuid;

#[async_trait]
pub trait ApplicationStoreEngine: Send + Sync {
    /// List all active applications for a given system
    async fn list_applications(&self, system_slug: &str) -> AppResult<Vec<ApplicationEntity>>;

    /// Get an application by its UUID
    async fn get_application_by_id(&self, id: Uuid) -> AppResult<ApplicationEntity>;

    /// Find an active application matching a bearer token
    async fn find_by_token(&self, token: &str) -> AppResult<Option<ApplicationEntity>>;

    /// Check if a system has any active applications configured
    async fn has_applications(&self, system_slug: &str) -> AppResult<bool>;

    /// Create a new application
    async fn create_application(
        &self,
        system_slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: serde_json::Value,
    ) -> AppResult<ApplicationEntity>;

    /// Update an existing application's details or permissions
    async fn update_application(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
        permissions: Option<serde_json::Value>,
    ) -> AppResult<ApplicationEntity>;

    /// Regenerate token for an application
    async fn regenerate_token(&self, id: Uuid) -> AppResult<ApplicationEntity>;

    /// Soft-delete an application
    async fn delete_application(&self, id: Uuid) -> AppResult<()>;
}

use crate::entities::{ModelEntity, ModelFieldEntity};
use async_trait::async_trait;
use foundry_core::error::AppResult;
use serde_json::Value;

#[async_trait]
pub trait ModelStoreEngine: Send + Sync {
    async fn list_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>>;

    async fn list_deleted_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>>;

    async fn get_model(&self, system_slug: &str, model_slug: &str) -> AppResult<ModelEntity>;

    async fn get_model_by_id(&self, model_id: i64) -> AppResult<ModelEntity>;

    async fn create_model(
        &self,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity>;

    async fn update_model(
        &self,
        model_id: i64,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity>;

    async fn list_fields(&self, model_id: i64) -> AppResult<Vec<ModelFieldEntity>>;

    async fn get_field(&self, field_id: i64) -> AppResult<ModelFieldEntity>;

    #[allow(clippy::too_many_arguments)]
    async fn add_field(
        &self,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity>;

    #[allow(clippy::too_many_arguments)]
    async fn update_field(
        &self,
        field_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity>;

    /// Soft delete a field
    async fn delete_field(&self, field_id: i64) -> AppResult<()>;

    /// Soft delete a dynamic data model by system_slug and model_slug
    async fn delete_model(&self, system_slug: &str, model_slug: &str) -> AppResult<()>;

    /// Soft delete a dynamic data model by id
    async fn delete_model_by_id(&self, model_id: i64) -> AppResult<()>;

    /// Restore a soft-deleted dynamic data model by id
    async fn restore_model(&self, model_id: i64) -> AppResult<ModelEntity>;
}

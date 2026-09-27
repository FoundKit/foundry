use crate::entities::{ModelEntity, ModelFieldEntity};
use async_trait::async_trait;
use foundry_core::error::AppResult;
use serde_json::Value;

#[async_trait]
pub trait ModelStoreEngine: Send + Sync {
    async fn list_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>>;

    async fn get_model(&self, system_slug: &str, model_slug: &str) -> AppResult<ModelEntity>;

    async fn create_model(
        &self,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity>;

    async fn list_fields(&self, model_id: i64) -> AppResult<Vec<ModelFieldEntity>>;

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
}

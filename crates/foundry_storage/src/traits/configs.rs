use crate::entities::SystemConfigEntity;
use async_trait::async_trait;
use foundry_core::error::AppResult;
use serde_json::Value;

#[async_trait]
pub trait ConfigStoreEngine: Send + Sync {
    /// Retrieve all config entities for a sub-system ordered by sort_order
    async fn list(&self, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>>;

    /// Retrieve aggregated configuration key-value map as a single JSON object
    async fn get_aggregated(&self, system_slug: &str) -> AppResult<Value>;

    /// Upsert a single config definition
    #[allow(clippy::too_many_arguments)]
    async fn upsert(
        &self,
        system_slug: &str,
        key: &str,
        label: &str,
        value_type: &str,
        value: Option<Value>,
        options: Value,
        sort_order: i32,
    ) -> AppResult<SystemConfigEntity>;

    /// Batch update multiple config values from a JSON key-value map
    async fn update_values(
        &self,
        system_slug: &str,
        values: &serde_json::Map<String, Value>,
    ) -> AppResult<()>;
}

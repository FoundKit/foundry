use crate::entities::SystemConfigEntity;
use crate::facade::Database;
use foundry_core::error::AppResult;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct ConfigStore;

impl ConfigStore {
    pub async fn list(db: &Database, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>> {
        db.configs().list(system_slug).await
    }

    pub async fn get_aggregated(db: &Database, system_slug: &str) -> AppResult<Value> {
        db.configs().get_aggregated(system_slug).await
    }

    pub async fn get_typed<T: DeserializeOwned>(db: &Database, system_slug: &str) -> AppResult<T> {
        db.get_typed_config(system_slug).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        db: &Database,
        system_slug: &str,
        key: &str,
        label: &str,
        value_type: &str,
        value: Option<Value>,
        options: Value,
        sort_order: i32,
    ) -> AppResult<SystemConfigEntity> {
        db.configs()
            .upsert(
                system_slug,
                key,
                label,
                value_type,
                value,
                options,
                sort_order,
            )
            .await
    }

    pub async fn update_values(
        db: &Database,
        system_slug: &str,
        values: &serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        db.configs().update_values(system_slug, values).await
    }
}

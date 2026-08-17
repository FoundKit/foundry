use crate::db::DbPool;
use crate::entities::SystemConfigEntity;
use foundry_core::error::{AppError, AppResult};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct ConfigStore;

impl ConfigStore {
    /// Retrieve all config entities for a sub-system ordered by sort_order
    pub async fn list(pool: &DbPool, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>> {
        let rows = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            SELECT id, system_id, key, label, value_type, value, options, sort_order, created_at, updated_at
            FROM system_configs
            WHERE system_id = $1
            ORDER BY sort_order ASC, id ASC
            "#
        )
        .bind(system_slug)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch configs: {}", e)))?;

        Ok(rows)
    }

    /// Retrieve aggregated configuration key-value map as a single JSON object
    pub async fn get_aggregated(pool: &DbPool, system_slug: &str) -> AppResult<Value> {
        let configs = Self::list(pool, system_slug).await?;
        let mut map = serde_json::Map::new();

        for cfg in configs {
            let val = cfg.value.unwrap_or(Value::Null);
            map.insert(cfg.key, val);
        }

        Ok(Value::Object(map))
    }

    /// Strongly-typed retrieval of sub-system configuration into a Rust struct
    pub async fn get_typed<T: DeserializeOwned>(pool: &DbPool, system_slug: &str) -> AppResult<T> {
        let agg = Self::get_aggregated(pool, system_slug).await?;
        serde_json::from_value::<T>(agg)
            .map_err(|e| AppError::Validation(format!("Failed to deserialize typed config: {}", e)))
    }

    /// Upsert a single config definition
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        pool: &DbPool,
        system_slug: &str,
        key: &str,
        label: &str,
        value_type: &str,
        value: Option<Value>,
        options: Value,
        sort_order: i32,
    ) -> AppResult<SystemConfigEntity> {
        let row = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            INSERT INTO system_configs (system_id, key, label, value_type, value, options, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (system_id, key)
            DO UPDATE SET
                label = EXCLUDED.label,
                value_type = EXCLUDED.value_type,
                value = EXCLUDED.value,
                options = EXCLUDED.options,
                sort_order = EXCLUDED.sort_order,
                updated_at = NOW()
            RETURNING id, system_id, key, label, value_type, value, options, sort_order, created_at, updated_at
            "#
        )
        .bind(system_slug)
        .bind(key)
        .bind(label)
        .bind(value_type)
        .bind(value)
        .bind(options)
        .bind(sort_order)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to upsert config: {}", e)))?;

        Ok(row)
    }

    /// Batch update multiple config values from a JSON key-value map
    pub async fn update_values(
        pool: &DbPool,
        system_slug: &str,
        values: &serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (k, v) in values {
            sqlx::query(
                r#"
                UPDATE system_configs
                SET value = $1, updated_at = NOW()
                WHERE system_id = $2 AND key = $3
                "#,
            )
            .bind(v)
            .bind(system_slug)
            .bind(k)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(format!("Failed to update config {}: {}", k, e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

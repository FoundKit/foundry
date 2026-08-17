use crate::db::DbPool;
use crate::entities::{ModelEntity, ModelFieldEntity, ModelRecordEntity};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use foundry_core::types::FieldType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct ModelStore;

impl ModelStore {
    pub async fn list_models(pool: &DbPool, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        let rows = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE system_id = $1 AND deleted_at IS NULL
            ORDER BY id ASC
            "#
        )
        .bind(system_slug)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list models: {}", e)))?;

        Ok(rows)
    }

    pub async fn get_model(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
    ) -> AppResult<ModelEntity> {
        let row = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE system_id = $1 AND slug = $2 AND deleted_at IS NULL
            "#
        )
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get model: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Model '{}' not found in system '{}'", model_slug, system_slug)))?;

        Ok(row)
    }

    pub async fn create_model(
        pool: &DbPool,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        let default_perms = serde_json::json!({
            "public_read": false,
            "public_write": false,
            "auth_read": true,
            "auth_write": false
        });
        let perms = permissions.unwrap_or(default_perms);

        let row = sqlx::query_as::<_, ModelEntity>(
            r#"
            INSERT INTO models (system_id, slug, name, description, permissions)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            "#
        )
        .bind(system_slug)
        .bind(slug)
        .bind(name)
        .bind(description)
        .bind(perms)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create model: {}", e)))?;

        Ok(row)
    }

    pub async fn list_fields(pool: &DbPool, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        let rows = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            SELECT id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            FROM model_fields
            WHERE model_id = $1
            ORDER BY sort_order ASC, id ASC
            "#
        )
        .bind(model_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list fields: {}", e)))?;

        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_field(
        pool: &DbPool,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        let opts = options.unwrap_or_else(|| serde_json::json!({}));
        let row = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            INSERT INTO model_fields (model_id, name, label, field_type, is_required, default_value, options, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            "#
        )
        .bind(model_id)
        .bind(name)
        .bind(label)
        .bind(field_type)
        .bind(is_required)
        .bind(default_value)
        .bind(opts)
        .bind(sort_order)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to add model field: {}", e)))?;

        Ok(row)
    }
}

/// Query parameters for listing dynamic records
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RecordQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

pub struct RecordStore;

impl RecordStore {
    /// In-memory validation of record payload against model fields
    pub fn validate_record(fields: &[ModelFieldEntity], data: &Value) -> AppResult<()> {
        let obj = data.as_object().ok_or_else(|| {
            AppError::Validation("Record payload must be a JSON object".to_string())
        })?;

        for field in fields {
            let val = obj.get(&field.name);
            if field.is_required && (val.is_none() || val == Some(&Value::Null)) {
                return Err(AppError::Validation(format!(
                    "Field '{}' ({}) is required",
                    field.name, field.label
                )));
            }

            if let Some(val) = val
                && !val.is_null()
                && let Ok(ft) = field.field_type.parse::<FieldType>()
            {
                match ft {
                    FieldType::String
                    | FieldType::Richtext
                    | FieldType::Image
                    | FieldType::File => {
                        if !val.is_string() {
                            return Err(AppError::Validation(format!(
                                "Field '{}' must be a string",
                                field.name
                            )));
                        }
                    }
                    FieldType::Integer => {
                        if !val.is_i64() && !val.is_u64() {
                            return Err(AppError::Validation(format!(
                                "Field '{}' must be an integer",
                                field.name
                            )));
                        }
                    }
                    FieldType::Number => {
                        if !val.is_number() {
                            return Err(AppError::Validation(format!(
                                "Field '{}' must be a number",
                                field.name
                            )));
                        }
                    }
                    FieldType::Boolean => {
                        if !val.is_boolean() {
                            return Err(AppError::Validation(format!(
                                "Field '{}' must be a boolean",
                                field.name
                            )));
                        }
                    }
                    FieldType::Array => {
                        if !val.is_array() {
                            return Err(AppError::Validation(format!(
                                "Field '{}' must be an array",
                                field.name
                            )));
                        }
                    }
                    FieldType::Datetime | FieldType::Relation => {
                        // Strings or numbers accepted
                    }
                }
            }
        }
        Ok(())
    }

    /// List records for a model with pagination
    pub async fn list(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let total_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM model_records
            WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Count query failed: {}", e)))?;

        let total = total_row.0 as u64;

        let records = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Fetch records failed: {}", e)))?;

        Ok(PaginatedData::new(records, page, page_size, total))
    }

    /// Get a single record by ID
    pub async fn get_by_id(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<ModelRecordEntity> {
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Get record failed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        Ok(record)
    }

    /// Create a new record
    pub async fn create(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            INSERT INTO model_records (system_id, model_slug, data)
            VALUES ($1, $2, $3)
            RETURNING id, system_id, model_slug, data, created_at, updated_at, deleted_at
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .bind(data)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Create record failed: {}", e)))?;

        Ok(record)
    }

    /// Update a record by ID
    pub async fn update(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            UPDATE model_records
            SET data = $1, updated_at = NOW()
            WHERE id = $2 AND system_id = $3 AND model_slug = $4 AND deleted_at IS NULL
            RETURNING id, system_id, model_slug, data, created_at, updated_at, deleted_at
            "#,
        )
        .bind(data)
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Update record failed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        Ok(record)
    }

    /// Soft delete a record
    pub async fn delete(
        pool: &DbPool,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE model_records
            SET deleted_at = NOW()
            WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Delete record failed: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_field(
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
    ) -> ModelFieldEntity {
        ModelFieldEntity {
            id: 1,
            model_id: 1,
            name: name.to_string(),
            label: label.to_string(),
            field_type: field_type.to_string(),
            is_required,
            default_value: None,
            options: serde_json::json!({}),
            sort_order: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_record_validation_success() {
        let fields = vec![
            sample_field("title", "Title", "string", true),
            sample_field("price", "Price", "number", true),
            sample_field("in_stock", "In Stock", "boolean", false),
            sample_field("tags", "Tags", "array", false),
        ];

        let valid_payload = serde_json::json!({
            "title": "Mechanical Keyboard",
            "price": 129.99,
            "in_stock": true,
            "tags": ["electronics", "gaming"]
        });

        assert!(RecordStore::validate_record(&fields, &valid_payload).is_ok());
    }

    #[test]
    fn test_record_validation_missing_required() {
        let fields = vec![sample_field("title", "Title", "string", true)];

        let missing_payload = serde_json::json!({
            "price": 100
        });

        assert!(RecordStore::validate_record(&fields, &missing_payload).is_err());
    }

    #[test]
    fn test_record_validation_type_mismatch() {
        let fields = vec![sample_field("price", "Price", "integer", true)];

        let wrong_type_payload = serde_json::json!({
            "price": "not_an_integer"
        });

        assert!(RecordStore::validate_record(&fields, &wrong_type_payload).is_err());
    }
}

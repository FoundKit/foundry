use crate::entities::{ModelEntity, ModelFieldEntity, ModelRecordEntity};
use crate::facade::Database;
pub use crate::traits::{RecordQuery, validate_record};
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;
use serde_json::Value;

pub struct ModelStore;

impl ModelStore {
    pub async fn list_models(db: &Database, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        db.models().list_models(system_slug).await
    }

    pub async fn get_model(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
    ) -> AppResult<ModelEntity> {
        db.models().get_model(system_slug, model_slug).await
    }

    pub async fn create_model(
        db: &Database,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        db.models()
            .create_model(system_slug, slug, name, description, permissions)
            .await
    }

    pub async fn list_fields(db: &Database, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        db.models().list_fields(model_id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_field(
        db: &Database,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        db.models()
            .add_field(
                model_id,
                name,
                label,
                field_type,
                is_required,
                default_value,
                options,
                sort_order,
            )
            .await
    }
}

pub struct RecordStore;

impl RecordStore {
    pub fn validate_record(fields: &[ModelFieldEntity], data: &Value) -> AppResult<()> {
        validate_record(fields, data)
    }

    pub async fn list(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>> {
        db.records().list(system_slug, model_slug, query).await
    }

    pub async fn get_by_id(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<ModelRecordEntity> {
        db.records().get_by_id(system_slug, model_slug, id).await
    }

    pub async fn create(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        db.records().create(system_slug, model_slug, data).await
    }

    pub async fn update(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        db.records().update(system_slug, model_slug, id, data).await
    }

    pub async fn delete(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<()> {
        db.records().delete(system_slug, model_slug, id).await
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

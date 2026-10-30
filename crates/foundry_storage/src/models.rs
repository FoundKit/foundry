pub use crate::entities::{
    ModelEntity, ModelExportItem, ModelFieldEntity, ModelRecordEntity, ReferencingFieldInfo,
};
use crate::facade::Database;
pub use crate::traits::{RecordQuery, coerce_record_data, validate_record};
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;
use serde_json::Value;

pub struct ModelStore;

impl ModelStore {
    pub async fn list_models(db: &Database, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        db.models().list_models(system_slug).await
    }

    pub async fn list_deleted_models(
        db: &Database,
        system_slug: &str,
    ) -> AppResult<Vec<ModelEntity>> {
        db.models().list_deleted_models(system_slug).await
    }

    pub async fn get_model(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
    ) -> AppResult<ModelEntity> {
        db.models().get_model(system_slug, model_slug).await
    }

    pub async fn get_model_by_id(db: &Database, model_id: i64) -> AppResult<ModelEntity> {
        db.models().get_model_by_id(model_id).await
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

    pub async fn update_model(
        db: &Database,
        model_id: i64,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        db.models()
            .update_model(model_id, slug, name, description, permissions)
            .await
    }

    pub async fn delete_model(db: &Database, system_slug: &str, model_slug: &str) -> AppResult<()> {
        db.models().delete_model(system_slug, model_slug).await
    }

    pub async fn delete_model_by_id(db: &Database, model_id: i64) -> AppResult<()> {
        db.models().delete_model_by_id(model_id).await
    }

    pub async fn find_referencing_fields(
        db: &Database,
        system_slug: &str,
        model_slug: &str,
        exclude_model_id: i64,
    ) -> AppResult<Vec<ReferencingFieldInfo>> {
        db.models()
            .find_referencing_fields(system_slug, model_slug, exclude_model_id)
            .await
    }

    pub async fn restore_model(db: &Database, model_id: i64) -> AppResult<ModelEntity> {
        db.models().restore_model(model_id).await
    }

    pub async fn list_fields(db: &Database, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        db.models().list_fields(model_id).await
    }

    pub async fn get_field(db: &Database, field_id: i64) -> AppResult<ModelFieldEntity> {
        db.models().get_field(field_id).await
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

    #[allow(clippy::too_many_arguments)]
    pub async fn update_field(
        db: &Database,
        field_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        db.models()
            .update_field(
                field_id,
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

    pub async fn delete_field(db: &Database, field_id: i64) -> AppResult<()> {
        db.models().delete_field(field_id).await
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
            deleted_at: None,
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

    #[test]
    fn test_coerce_record_data_type_switching() {
        let fields = vec![
            sample_field("num_to_str", "Num to Str", "string", true),
            sample_field("str_to_num", "Str to Num", "number", true),
            sample_field("invalid_to_num", "Invalid to Num", "number", true),
            sample_field("str_to_int", "Str to Int", "integer", true),
            sample_field("invalid_to_int", "Invalid to Int", "integer", true),
            sample_field("num_to_bool", "Num to Bool", "boolean", true),
            sample_field("str_to_bool", "Str to Bool", "boolean", true),
            sample_field("rel_to_str", "Rel to Str", "string", true),
            sample_field("rel_to_num", "Rel to Num", "number", true),
            sample_field("rel_to_bool", "Rel to Bool", "boolean", true),
            sample_field("str_to_rel", "Str to Rel", "relation", false),
            sample_field("invalid_to_rel", "Invalid to Rel", "relation", false),
            sample_field("str_to_arr", "Str to Arr", "array", false),
            sample_field("json_str_to_arr", "JSON Str to Arr", "array", false),
        ];

        let mut data = serde_json::json!({
            "num_to_str": 12345,
            "str_to_num": "456.78",
            "invalid_to_num": "not_a_number",
            "str_to_int": "987",
            "invalid_to_int": "not_an_int",
            "num_to_bool": 1,
            "str_to_bool": "yes",
            "rel_to_str": 8888, // previously relation stored record id 8888
            "rel_to_num": 8888,
            "rel_to_bool": 8888,
            "str_to_rel": "9999",
            "invalid_to_rel": "xyz",
            "str_to_arr": "simple_tag",
            "json_str_to_arr": "[\"alpha\", \"beta\"]",
        });

        coerce_record_data(&fields, &mut data);

        assert_eq!(data["num_to_str"], "12345");
        assert_eq!(data["str_to_num"], 456.78);
        assert_eq!(data["invalid_to_num"], 0);
        assert_eq!(data["str_to_int"], 987);
        assert_eq!(data["invalid_to_int"], 0);
        assert_eq!(data["num_to_bool"], true);
        assert_eq!(data["str_to_bool"], true);
        assert_eq!(data["rel_to_str"], "8888");
        assert_eq!(data["rel_to_num"], 8888);
        assert_eq!(data["rel_to_bool"], true);
        assert_eq!(data["str_to_rel"], 9999);
        assert_eq!(data["invalid_to_rel"], serde_json::Value::Null);
        assert_eq!(data["str_to_arr"], serde_json::json!(["simple_tag"]));
        assert_eq!(
            data["json_str_to_arr"],
            serde_json::json!(["alpha", "beta"])
        );
    }
}

use crate::entities::{ModelFieldEntity, ModelRecordEntity};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use foundry_core::types::FieldType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Query parameters for listing dynamic records
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RecordQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

/// In-memory validation of record payload against model fields
pub fn validate_record(fields: &[ModelFieldEntity], data: &Value) -> AppResult<()> {
    let obj = data
        .as_object()
        .ok_or_else(|| AppError::Validation("Record payload must be a JSON object".to_string()))?;

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
                FieldType::String | FieldType::Richtext | FieldType::Image | FieldType::File => {
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

#[async_trait]
pub trait RecordStoreEngine: Send + Sync {
    async fn list(
        &self,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>>;

    async fn get_by_id(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<ModelRecordEntity>;

    async fn create(
        &self,
        system_slug: &str,
        model_slug: &str,
        data: Value,
    ) -> AppResult<ModelRecordEntity>;

    async fn update(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity>;

    async fn delete(&self, system_slug: &str, model_slug: &str, id: i64) -> AppResult<()>;
}

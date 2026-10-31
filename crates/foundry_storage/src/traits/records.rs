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
    pub fields: Option<String>,
    pub exclude_fields: Option<String>,
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
                FieldType::Relation => {
                    let is_valid_id = if val.is_i64() || val.is_u64() {
                        true
                    } else if let Some(s) = val.as_str() {
                        s.trim().parse::<i64>().is_ok()
                    } else {
                        false
                    };
                    if !is_valid_id {
                        return Err(AppError::Validation(format!(
                            "Field '{}' ({}) must be a valid target record ID",
                            field.name, field.label
                        )));
                    }
                }
                FieldType::Datetime => {
                    // Strings or numbers accepted
                }
            }
        }
    }
    Ok(())
}

/// Gracefully coerces record JSON data according to the current model schema.
/// This guarantees backward compatibility when field types are altered.
/// Read queries coerce in-memory and do not alter the persistent storage
/// until explicit admin or API update writes occur.
pub fn coerce_record_data(fields: &[ModelFieldEntity], data: &mut Value) {
    let Some(obj) = data.as_object_mut() else {
        return;
    };

    for field in fields {
        let Ok(ft) = field.field_type.parse::<FieldType>() else {
            continue;
        };

        if let Some(val) = obj.get_mut(&field.name) {
            if val.is_null() {
                if field.is_required {
                    *val = match ft {
                        FieldType::String
                        | FieldType::Richtext
                        | FieldType::Image
                        | FieldType::File => Value::String(String::new()),
                        FieldType::Integer => Value::Number(0.into()),
                        FieldType::Number => serde_json::Number::from_f64(0.0)
                            .map(Value::Number)
                            .unwrap_or(Value::Number(0.into())),
                        FieldType::Boolean => Value::Bool(false),
                        FieldType::Array => Value::Array(Vec::new()),
                        FieldType::Datetime | FieldType::Relation => Value::Null,
                    };
                }
            } else {
                match ft {
                    FieldType::String
                    | FieldType::Richtext
                    | FieldType::Image
                    | FieldType::File => {
                        if !val.is_string() {
                            *val = match val {
                                Value::Number(n) => Value::String(n.to_string()),
                                Value::Bool(b) => Value::String(b.to_string()),
                                Value::Array(_) | Value::Object(_) => {
                                    Value::String(val.to_string())
                                }
                                _ => Value::String(String::new()),
                            };
                        }
                    }
                    FieldType::Integer => {
                        if !val.is_i64() && !val.is_u64() {
                            *val = if let Some(f) = val.as_f64() {
                                Value::Number((f as i64).into())
                            } else if let Some(s) = val.as_str() {
                                if let Ok(i) = s.trim().parse::<i64>() {
                                    Value::Number(i.into())
                                } else if let Ok(f) = s.trim().parse::<f64>() {
                                    Value::Number((f as i64).into())
                                } else {
                                    Value::Number(0.into())
                                }
                            } else if let Some(b) = val.as_bool() {
                                Value::Number((if b { 1 } else { 0 }).into())
                            } else {
                                Value::Number(0.into())
                            };
                        }
                    }
                    FieldType::Number => {
                        if !val.is_number() {
                            *val = if let Some(s) = val.as_str() {
                                if let Ok(f) = s.trim().parse::<f64>() {
                                    serde_json::Number::from_f64(f)
                                        .map(Value::Number)
                                        .unwrap_or(Value::Number(0.into()))
                                } else {
                                    Value::Number(0.into())
                                }
                            } else if let Some(b) = val.as_bool() {
                                Value::Number((if b { 1 } else { 0 }).into())
                            } else {
                                Value::Number(0.into())
                            };
                        }
                    }
                    FieldType::Boolean => {
                        if !val.is_boolean() {
                            *val = if let Some(n) = val.as_i64() {
                                Value::Bool(n != 0)
                            } else if let Some(u) = val.as_u64() {
                                Value::Bool(u != 0)
                            } else if let Some(f) = val.as_f64() {
                                Value::Bool(f != 0.0)
                            } else if let Some(s) = val.as_str() {
                                match s.trim().to_ascii_lowercase().as_str() {
                                    "true" | "1" | "yes" | "y" | "t" => Value::Bool(true),
                                    _ => Value::Bool(false),
                                }
                            } else {
                                Value::Bool(false)
                            };
                        }
                    }
                    FieldType::Array => {
                        if !val.is_array() {
                            *val = if let Some(s) = val.as_str() {
                                if let Ok(parsed) = serde_json::from_str::<Value>(s)
                                    && parsed.is_array()
                                {
                                    parsed
                                } else {
                                    Value::Array(vec![Value::String(s.to_string())])
                                }
                            } else {
                                Value::Array(vec![val.clone()])
                            };
                        }
                    }
                    FieldType::Relation => {
                        if !val.is_i64() && !val.is_u64() {
                            *val = if let Some(f) = val.as_f64() {
                                Value::Number((f as i64).into())
                            } else if let Some(s) = val.as_str() {
                                if let Ok(i) = s.trim().parse::<i64>() {
                                    Value::Number(i.into())
                                } else {
                                    Value::Null
                                }
                            } else {
                                Value::Null
                            };
                        }
                    }
                    FieldType::Datetime => {
                        // Accept as is
                    }
                }
            }
        } else if field.is_required {
            let default_val = match ft {
                FieldType::String | FieldType::Richtext | FieldType::Image | FieldType::File => {
                    Value::String(String::new())
                }
                FieldType::Integer => Value::Number(0.into()),
                FieldType::Number => serde_json::Number::from_f64(0.0)
                    .map(Value::Number)
                    .unwrap_or(Value::Number(0.into())),
                FieldType::Boolean => Value::Bool(false),
                FieldType::Array => Value::Array(Vec::new()),
                FieldType::Datetime | FieldType::Relation => Value::Null,
            };
            obj.insert(field.name.clone(), default_val);
        }
    }
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

use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_core::types::is_valid_slug;
use foundry_storage::{ModelEntity, ModelExportItem, ModelFieldEntity};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct AddModelFieldRequest {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub is_required: Option<bool>,
    pub default_value: Option<Value>,
    pub options: Option<Value>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelFieldRequest {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub is_required: Option<bool>,
    pub default_value: Option<Value>,
    pub options: Option<Value>,
    pub sort_order: Option<i32>,
}

fn is_valid_field_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 48 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub async fn list_models_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<ModelEntity>>>> {
    let models = state.db.models().list_models(&system_slug).await?;
    Ok(Json(ApiResponse::success(models)))
}

pub async fn list_deleted_models_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<ModelEntity>>>> {
    let models = state.db.models().list_deleted_models(&system_slug).await?;
    Ok(Json(ApiResponse::success(models)))
}

pub async fn create_model_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(payload): Json<CreateModelRequest>,
) -> AppResult<Json<ApiResponse<ModelEntity>>> {
    if !is_valid_slug(&payload.slug, 48) {
        return Err(AppError::Validation(
            "Model slug must be 2-48 lowercase alphanumeric characters or underscore/hyphen"
                .to_string(),
        ));
    }

    let model = state
        .db
        .models()
        .create_model(
            &system_slug,
            &payload.slug,
            &payload.name,
            payload.description.as_deref(),
            payload.permissions,
        )
        .await?;

    Ok(Json(ApiResponse::success(model)))
}

pub async fn update_model_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
    Json(payload): Json<UpdateModelRequest>,
) -> AppResult<Json<ApiResponse<ModelEntity>>> {
    if !is_valid_slug(&payload.slug, 48) {
        return Err(AppError::Validation(
            "Model slug must be 2-48 lowercase alphanumeric characters or underscore/hyphen"
                .to_string(),
        ));
    }

    let updated = state
        .db
        .models()
        .update_model(
            model_id,
            &payload.slug,
            &payload.name,
            payload.description.as_deref(),
            payload.permissions,
        )
        .await?;

    Ok(Json(ApiResponse::success(updated)))
}

pub async fn delete_model_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    state.db.models().delete_model_by_id(model_id).await?;
    Ok(Json(ApiResponse::success(())))
}

pub async fn restore_model_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
) -> AppResult<Json<ApiResponse<ModelEntity>>> {
    let restored = state.db.models().restore_model(model_id).await?;
    Ok(Json(ApiResponse::success(restored)))
}

pub async fn list_model_fields_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
) -> AppResult<Json<ApiResponse<Vec<ModelFieldEntity>>>> {
    let fields = state.db.models().list_fields(model_id).await?;
    Ok(Json(ApiResponse::success(fields)))
}

pub async fn add_model_field_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
    Json(payload): Json<AddModelFieldRequest>,
) -> AppResult<Json<ApiResponse<ModelFieldEntity>>> {
    if !is_valid_field_name(&payload.name) {
        return Err(AppError::Validation(
            "Field name must be 1-48 lowercase alphanumeric characters or underscores".to_string(),
        ));
    }

    let is_required = payload.is_required.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);

    let field = state
        .db
        .models()
        .add_field(
            model_id,
            &payload.name,
            &payload.label,
            &payload.field_type,
            is_required,
            payload.default_value,
            payload.options,
            sort_order,
        )
        .await?;

    Ok(Json(ApiResponse::success(field)))
}

pub async fn update_model_field_handler(
    State(state): State<AppState>,
    Path((_system_slug, _model_id, field_id)): Path<(String, i64, i64)>,
    Json(payload): Json<UpdateModelFieldRequest>,
) -> AppResult<Json<ApiResponse<ModelFieldEntity>>> {
    if !is_valid_field_name(&payload.name) {
        return Err(AppError::Validation(
            "Field name must be 1-48 lowercase alphanumeric characters or underscores".to_string(),
        ));
    }

    let is_required = payload.is_required.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);

    let updated = state
        .db
        .models()
        .update_field(
            field_id,
            &payload.name,
            &payload.label,
            &payload.field_type,
            is_required,
            payload.default_value,
            payload.options,
            sort_order,
        )
        .await?;

    Ok(Json(ApiResponse::success(updated)))
}

pub async fn delete_model_field_handler(
    State(state): State<AppState>,
    Path((_system_slug, _model_id, field_id)): Path<(String, i64, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    state.db.models().delete_field(field_id).await?;
    Ok(Json(ApiResponse::success(())))
}

#[derive(Debug, Deserialize, Default)]
pub struct ExportModelsPayload {
    pub model_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
pub struct ImportModelsResult {
    pub total: usize,
    pub created: usize,
    pub updated: usize,
    pub restored: usize,
    pub models: Vec<ModelEntity>,
}

async fn export_models_impl(
    state: AppState,
    system_slug: String,
    model_ids: Option<Vec<i64>>,
) -> AppResult<Json<ApiResponse<Vec<ModelExportItem>>>> {
    let all_models = state.db.models().list_models(&system_slug).await?;
    let target_models: Vec<ModelEntity> = if let Some(ids) = model_ids {
        if !ids.is_empty() {
            let id_set: std::collections::HashSet<i64> = ids.into_iter().collect();
            all_models
                .into_iter()
                .filter(|m| id_set.contains(&m.id))
                .collect()
        } else {
            all_models
        }
    } else {
        all_models
    };

    let mut export_items = Vec::with_capacity(target_models.len());
    for model in target_models {
        let fields = state.db.models().list_fields(model.id).await?;
        export_items.push(ModelExportItem::from_entity(model, fields));
    }

    Ok(Json(ApiResponse::success(export_items)))
}

pub async fn export_models_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(payload): Json<ExportModelsPayload>,
) -> AppResult<Json<ApiResponse<Vec<ModelExportItem>>>> {
    export_models_impl(state, system_slug, payload.model_ids).await
}

pub async fn export_models_get_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<ModelExportItem>>>> {
    export_models_impl(state, system_slug, None).await
}

pub async fn import_models_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(items): Json<Vec<ModelExportItem>>,
) -> AppResult<Json<ApiResponse<ImportModelsResult>>> {
    if items.is_empty() {
        return Err(AppError::Validation(
            "No models provided for import".to_string(),
        ));
    }

    let mut created_count = 0;
    let mut updated_count = 0;
    let mut restored_count = 0;
    let mut result_models = Vec::with_capacity(items.len());

    for item in items {
        if !is_valid_slug(&item.slug, 48) {
            return Err(AppError::Validation(format!(
                "Invalid model slug '{}': must be 2-48 lowercase alphanumeric characters or underscore/hyphen",
                item.slug
            )));
        }

        let existing_active = state.db.models().list_models(&system_slug).await?;
        let existing_deleted = state.db.models().list_deleted_models(&system_slug).await?;

        let existing_model = existing_active
            .into_iter()
            .find(|m| m.slug == item.slug)
            .or_else(|| existing_deleted.into_iter().find(|m| m.slug == item.slug));

        let model = if let Some(existing) = existing_model {
            if existing.deleted_at.is_some() {
                state.db.models().restore_model(existing.id).await?;
                restored_count += 1;
            }
            let updated = state
                .db
                .models()
                .update_model(
                    existing.id,
                    &item.slug,
                    &item.name,
                    item.description.as_deref(),
                    Some(item.permissions),
                )
                .await?;
            updated_count += 1;
            updated
        } else {
            let created = state
                .db
                .models()
                .create_model(
                    &system_slug,
                    &item.slug,
                    &item.name,
                    item.description.as_deref(),
                    Some(item.permissions),
                )
                .await?;
            created_count += 1;
            created
        };

        // Sync fields for this model
        let current_fields = state.db.models().list_fields(model.id).await?;
        for field in item.fields {
            if !is_valid_field_name(&field.name) {
                continue;
            }

            let mut options = field.options;
            // If this is a relation field, rewrite target_system if it pointed to the source system
            if field.field_type == "relation"
                && let Some(opt_obj) = options.as_object_mut()
            {
                let source_target_sys = opt_obj
                    .get("target_system")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if source_target_sys.is_empty() || source_target_sys == item.system_id {
                    opt_obj.insert("target_system".to_string(), serde_json::json!(&system_slug));
                }
            }

            let matched_field = current_fields.iter().find(|f| f.name == field.name);
            if let Some(existing_f) = matched_field {
                state
                    .db
                    .models()
                    .update_field(
                        existing_f.id,
                        &field.name,
                        &field.label,
                        &field.field_type,
                        field.is_required,
                        field.default_value,
                        Some(options),
                        field.sort_order,
                    )
                    .await?;
            } else {
                state
                    .db
                    .models()
                    .add_field(
                        model.id,
                        &field.name,
                        &field.label,
                        &field.field_type,
                        field.is_required,
                        field.default_value,
                        Some(options),
                        field.sort_order,
                    )
                    .await?;
            }
        }

        result_models.push(model);
    }

    Ok(Json(ApiResponse::success(ImportModelsResult {
        total: result_models.len(),
        created: created_count,
        updated: updated_count,
        restored: restored_count,
        models: result_models,
    })))
}

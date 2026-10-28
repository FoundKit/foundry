use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_core::types::is_valid_slug;
use foundry_storage::{ModelEntity, ModelFieldEntity};
use serde::Deserialize;
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

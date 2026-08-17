use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_core::types::is_valid_slug;
use foundry_storage::{ModelEntity, ModelFieldEntity, ModelStore};
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
pub struct AddModelFieldRequest {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub is_required: Option<bool>,
    pub default_value: Option<Value>,
    pub options: Option<Value>,
    pub sort_order: Option<i32>,
}

pub async fn list_models_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<ModelEntity>>>> {
    let models = ModelStore::list_models(&state.db, &system_slug).await?;
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

    let model = ModelStore::create_model(
        &state.db,
        &system_slug,
        &payload.slug,
        &payload.name,
        payload.description.as_deref(),
        payload.permissions,
    )
    .await?;

    Ok(Json(ApiResponse::success(model)))
}

pub async fn list_model_fields_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
) -> AppResult<Json<ApiResponse<Vec<ModelFieldEntity>>>> {
    let fields = ModelStore::list_fields(&state.db, model_id).await?;
    Ok(Json(ApiResponse::success(fields)))
}

pub async fn add_model_field_handler(
    State(state): State<AppState>,
    Path((_system_slug, model_id)): Path<(String, i64)>,
    Json(payload): Json<AddModelFieldRequest>,
) -> AppResult<Json<ApiResponse<ModelFieldEntity>>> {
    let is_required = payload.is_required.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);

    let field = ModelStore::add_field(
        &state.db,
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

use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_storage::{ConfigStore, SystemConfigEntity};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct UpsertConfigSchemaRequest {
    pub key: String,
    pub label: String,
    pub value_type: String,
    pub default_value: Option<Value>,
    pub options: Option<Value>,
    pub sort_order: Option<i32>,
}

/// GET /api/v1/s/:system_slug/configs (Aggregated key-value JSON)
pub async fn get_aggregated_configs_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Value>>> {
    let configs = ConfigStore::get_aggregated(&state.db, &system_slug).await?;
    Ok(Json(ApiResponse::success(configs)))
}

/// PUT /api/v1/s/:system_slug/configs (Batch update config values)
pub async fn update_aggregated_configs_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(payload): Json<Value>,
) -> AppResult<Json<ApiResponse<()>>> {
    let map = payload.as_object().ok_or_else(|| {
        AppError::Validation(
            "Payload must be a key-value object of configuration values".to_string(),
        )
    })?;

    ConfigStore::update_values(&state.db, &system_slug, map).await?;
    Ok(Json(ApiResponse::success(())))
}

/// GET /api/v1/admin/s/:system_slug/configs/schema (List config item specifications)
pub async fn list_config_schema_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<SystemConfigEntity>>>> {
    let list = ConfigStore::list(&state.db, &system_slug).await?;
    Ok(Json(ApiResponse::success(list)))
}

/// POST /api/v1/admin/s/:system_slug/configs/schema (Define or update a config item specification)
pub async fn upsert_config_schema_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(payload): Json<UpsertConfigSchemaRequest>,
) -> AppResult<Json<ApiResponse<SystemConfigEntity>>> {
    let options = payload.options.unwrap_or_else(|| serde_json::json!({}));
    let sort_order = payload.sort_order.unwrap_or(0);

    let config = ConfigStore::upsert(
        &state.db,
        &system_slug,
        &payload.key,
        &payload.label,
        &payload.value_type,
        payload.default_value,
        options,
        sort_order,
    )
    .await?;

    Ok(Json(ApiResponse::success(config)))
}

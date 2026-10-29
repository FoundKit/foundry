use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_storage::ApplicationEntity;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
    pub permissions: Option<Value>,
}

/// GET /api/admin/s/:system_slug/apps
pub async fn list_applications_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<ApplicationEntity>>>> {
    let list = state.db.apps().list_applications(&system_slug).await?;
    Ok(Json(ApiResponse::success(list)))
}

/// POST /api/admin/s/:system_slug/apps
pub async fn create_application_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
    Json(payload): Json<CreateApplicationRequest>,
) -> AppResult<Json<ApiResponse<ApplicationEntity>>> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "Application name cannot be empty".to_string(),
        ));
    }

    let default_perms = serde_json::json!({
        "configs": { "read": true, "write": false },
        "models": {}
    });
    let permissions = payload.permissions.unwrap_or(default_perms);

    let app = state
        .db
        .apps()
        .create_application(
            &system_slug,
            name,
            payload.description.as_deref(),
            permissions,
        )
        .await?;

    Ok(Json(ApiResponse::success(app)))
}

/// GET /api/admin/s/:system_slug/apps/:id
pub async fn get_application_handler(
    State(state): State<AppState>,
    Path((system_slug, id)): Path<(String, Uuid)>,
) -> AppResult<Json<ApiResponse<ApplicationEntity>>> {
    let app = state.db.apps().get_application_by_id(id).await?;
    if app.system_id != system_slug {
        return Err(AppError::NotFound(format!(
            "Application '{}' not found in system '{}'",
            id, system_slug
        )));
    }
    Ok(Json(ApiResponse::success(app)))
}

/// PUT /api/admin/s/:system_slug/apps/:id
pub async fn update_application_handler(
    State(state): State<AppState>,
    Path((system_slug, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateApplicationRequest>,
) -> AppResult<Json<ApiResponse<ApplicationEntity>>> {
    let app = state.db.apps().get_application_by_id(id).await?;
    if app.system_id != system_slug {
        return Err(AppError::NotFound(format!(
            "Application '{}' not found in system '{}'",
            id, system_slug
        )));
    }

    let trimmed_name = payload.name.as_deref().map(|s| s.trim());
    if let Some(name) = trimmed_name
        && name.is_empty()
    {
        return Err(AppError::Validation(
            "Application name cannot be empty".to_string(),
        ));
    }

    let updated = state
        .db
        .apps()
        .update_application(
            id,
            trimmed_name,
            payload.description.as_deref(),
            payload.status,
            payload.permissions,
        )
        .await?;

    Ok(Json(ApiResponse::success(updated)))
}

/// POST /api/admin/s/:system_slug/apps/:id/regenerate-secret
pub async fn regenerate_app_secret_handler(
    State(state): State<AppState>,
    Path((system_slug, id)): Path<(String, Uuid)>,
) -> AppResult<Json<ApiResponse<ApplicationEntity>>> {
    let app = state.db.apps().get_application_by_id(id).await?;
    if app.system_id != system_slug {
        return Err(AppError::NotFound(format!(
            "Application '{}' not found in system '{}'",
            id, system_slug
        )));
    }

    let updated = state.db.apps().regenerate_token(id).await?;
    Ok(Json(ApiResponse::success(updated)))
}

/// DELETE /api/admin/s/:system_slug/apps/:id
pub async fn delete_application_handler(
    State(state): State<AppState>,
    Path((system_slug, id)): Path<(String, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    let app = state.db.apps().get_application_by_id(id).await?;
    if app.system_id != system_slug {
        return Err(AppError::NotFound(format!(
            "Application '{}' not found in system '{}'",
            id, system_slug
        )));
    }

    state.db.apps().delete_application(id).await?;
    Ok(Json(ApiResponse::success(())))
}

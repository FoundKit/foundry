use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use foundry_auth::AdminClaims;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_core::types::is_valid_slug;
use foundry_storage::{SystemEntity, SystemStore};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSystemRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSystemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
}

pub async fn list_systems_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<Vec<SystemEntity>>>> {
    let all_systems = SystemStore::list(&state.db).await?;

    if claims.role == "super_admin" || claims.allowed_systems.iter().any(|s| s == "*") {
        return Ok(Json(ApiResponse::success(all_systems)));
    }

    let filtered = all_systems
        .into_iter()
        .filter(|s| claims.allowed_systems.contains(&s.slug))
        .collect();

    Ok(Json(ApiResponse::success(filtered)))
}

pub async fn create_system_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<CreateSystemRequest>,
) -> AppResult<Json<ApiResponse<SystemEntity>>> {
    if claims.role != "super_admin" {
        return Err(AppError::Forbidden(
            "Only Super Admin can create sub-systems".to_string(),
        ));
    }

    if !is_valid_slug(&payload.slug, 32) {
        return Err(AppError::Validation(
            "System slug must be 2-32 lowercase alphanumeric characters or underscore/hyphen"
                .to_string(),
        ));
    }

    let system = SystemStore::create(
        &state.db,
        &payload.slug,
        &payload.name,
        payload.description.as_deref(),
    )
    .await?;

    Ok(Json(ApiResponse::success(system)))
}

pub async fn update_system_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSystemRequest>,
) -> AppResult<Json<ApiResponse<SystemEntity>>> {
    if claims.role != "super_admin" {
        return Err(AppError::Forbidden(
            "Only Super Admin can update sub-system status".to_string(),
        ));
    }

    let system = SystemStore::update(
        &state.db,
        id,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.status,
    )
    .await?;

    Ok(Json(ApiResponse::success(system)))
}

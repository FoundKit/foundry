use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use foundry_auth::AdminClaims;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::{ApiResponse, PaginatedData};
use foundry_core::types::is_valid_slug;
use foundry_storage::{
    PlatformSummary, SystemEntity, SystemItem, SystemQuery, SystemStats, SystemStore,
};
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

/// GET /api/v1/admin/systems (Paginated and filterable list of sub-systems)
pub async fn list_systems_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Query(query): Query<SystemQuery>,
) -> AppResult<Json<ApiResponse<PaginatedData<SystemItem>>>> {
    let is_platform_wide = claims.has_platform_manage_access();
    let allowed = if is_platform_wide {
        None
    } else {
        Some(claims.allowed_systems.as_slice())
    };

    let result = SystemStore::list_paginated(&state.db, query, allowed).await?;
    Ok(Json(ApiResponse::success(result)))
}

/// GET /api/v1/admin/systems/{id} (Get single sub-system by UUID)
pub async fn get_system_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<SystemItem>>> {
    let system = SystemStore::get_by_id(&state.db, id).await?;

    let is_platform_wide = claims.has_platform_manage_access();
    if !is_platform_wide && !claims.allowed_systems.contains(&system.slug) {
        return Err(AppError::Forbidden(
            "Access denied: You are not authorized to view this sub-system".to_string(),
        ));
    }

    Ok(Json(ApiResponse::success(system)))
}

/// GET /api/v1/admin/s/{system_slug}/details (Get single sub-system by Slug)
pub async fn get_system_by_slug_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<SystemItem>>> {
    let system = SystemStore::get_by_slug(&state.db, &system_slug).await?;
    Ok(Json(ApiResponse::success(system)))
}

/// GET /api/v1/admin/s/{system_slug}/stats (Get detailed statistics for a sub-system)
pub async fn get_system_stats_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<SystemStats>>> {
    let stats = SystemStore::get_stats(&state.db, &system_slug).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// GET /api/v1/admin/platform/summary (Get platform-wide statistics summary)
pub async fn get_platform_summary_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<PlatformSummary>>> {
    if !claims.can_view_platform_summary() {
        return Err(AppError::Forbidden(
            "Permission denied: Topic Admin cannot view platform-wide summary statistics"
                .to_string(),
        ));
    }

    let summary = SystemStore::platform_summary(&state.db).await?;
    Ok(Json(ApiResponse::success(summary)))
}

/// POST /api/v1/admin/systems (Create sub-system)
pub async fn create_system_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<CreateSystemRequest>,
) -> AppResult<Json<ApiResponse<SystemEntity>>> {
    if !claims.has_platform_manage_access() {
        return Err(AppError::Forbidden(
            "Permission denied: Topic Admin cannot create sub-systems".to_string(),
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

/// PUT /api/v1/admin/systems/{id} (Update sub-system)
pub async fn update_system_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSystemRequest>,
) -> AppResult<Json<ApiResponse<SystemEntity>>> {
    if !claims.has_platform_manage_access() {
        return Err(AppError::Forbidden(
            "Permission denied: Topic Admin cannot update sub-systems".to_string(),
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

/// GET /admin-api/v1/s/{system_slug}/custom-pages (List custom admin pages registered for a sub-system)
pub async fn list_subsystem_custom_pages_handler(
    State(state): State<AppState>,
    Path(system_slug): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<foundry_core::CustomAdminPageSpec>>>> {
    let pages = state
        .subsystems
        .iter()
        .find(|s| s.slug() == system_slug)
        .map(|s| s.custom_admin_pages())
        .unwrap_or_default();

    Ok(Json(ApiResponse::success(pages)))
}


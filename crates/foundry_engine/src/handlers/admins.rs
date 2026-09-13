use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use foundry_auth::{AdminClaims, hash_password};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_storage::{AdminEntity, AdminStore};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAdminRequest {
    #[validate(length(min = 2, max = 48))]
    pub username: String,
    pub email: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    pub role: String, // "super_admin", "admin", "topic_admin"
    pub allowed_systems: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAdminRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub allowed_systems: Option<Vec<String>>,
    pub status: Option<i16>,
}

pub async fn list_admins_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<Vec<AdminEntity>>>> {
    if !claims.can_manage_admins() {
        return Err(AppError::Forbidden(
            "Only Super Admin can view all administrators".to_string(),
        ));
    }

    let admins = AdminStore::list(&state.db).await?;
    Ok(Json(ApiResponse::success(admins)))
}

pub async fn create_admin_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<CreateAdminRequest>,
) -> AppResult<Json<ApiResponse<AdminEntity>>> {
    if !claims.can_manage_admins() {
        return Err(AppError::Forbidden(
            "Only Super Admin can create administrators".to_string(),
        ));
    }

    payload.validate()?;

    let normalized_role = payload.role.trim().to_lowercase();
    if !matches!(
        normalized_role.as_str(),
        "super_admin" | "admin" | "topic_admin"
    ) {
        return Err(AppError::Validation(
            "Role must be 'super_admin' (超级管理员), 'admin' (普通管理员), or 'topic_admin' (专题管理员)"
                .to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;

    // Super Admin and General Admin have wildcard platform-wide subsystem access;
    // Topic Admin has explicit scoped subsystem access.
    let allowed: Vec<String> = match normalized_role.as_str() {
        "super_admin" | "admin" => vec!["*".to_string()],
        "topic_admin" => payload.allowed_systems.unwrap_or_default(),
        _ => vec![],
    };
    let allowed_json = serde_json::to_value(allowed).unwrap_or_else(|_| serde_json::json!([]));

    let admin = AdminStore::create(
        &state.db,
        &payload.username,
        payload.email.as_deref(),
        &password_hash,
        &normalized_role,
        allowed_json,
    )
    .await?;

    Ok(Json(ApiResponse::success(admin)))
}

pub async fn update_admin_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAdminRequest>,
) -> AppResult<Json<ApiResponse<AdminEntity>>> {
    if !claims.can_manage_admins() {
        return Err(AppError::Forbidden(
            "Only Super Admin can update administrators".to_string(),
        ));
    }

    let password_hash = match payload.password {
        Some(ref pwd) => Some(hash_password(pwd)?),
        None => None,
    };

    let normalized_role = payload.role.as_ref().map(|r| r.trim().to_lowercase());
    if let Some(r) = normalized_role.as_deref() {
        match r {
            "super_admin" | "admin" | "topic_admin" => {}
            _ => {
                return Err(AppError::Validation(
                    "Role must be 'super_admin' (超级管理员), 'admin' (普通管理员), or 'topic_admin' (专题管理员)"
                        .to_string(),
                ));
            }
        }
    }

    let allowed_json = match normalized_role.as_deref() {
        Some("super_admin") | Some("admin") => Some(serde_json::json!(["*"])),
        Some("topic_admin") => {
            let allowed = payload.allowed_systems.unwrap_or_default();
            Some(serde_json::to_value(allowed).unwrap_or_else(|_| serde_json::json!([])))
        }
        None => payload
            .allowed_systems
            .map(|systems| serde_json::to_value(systems).unwrap_or_else(|_| serde_json::json!([]))),
        _ => None,
    };

    let admin = AdminStore::update(
        &state.db,
        id,
        payload.email.as_deref(),
        password_hash.as_deref(),
        normalized_role.as_deref(),
        allowed_json,
        payload.status,
    )
    .await?;

    Ok(Json(ApiResponse::success(admin)))
}

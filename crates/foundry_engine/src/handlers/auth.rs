use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, State},
};
use foundry_auth::{AdminClaims, verify_password};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use foundry_storage::AdminStore;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: AdminProfile,
}

#[derive(Debug, Serialize)]
pub struct AdminProfile {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub allowed_systems: serde_json::Value,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    payload.validate()?;

    let admin = AdminStore::get_by_username(&state.db, &payload.username)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if admin.status != 1 {
        return Err(AppError::Forbidden("Account has been disabled".to_string()));
    }

    let is_valid = verify_password(&payload.password, &admin.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let allowed: Vec<String> =
        serde_json::from_value(admin.allowed_systems.clone()).unwrap_or_default();

    let token = state
        .jwt
        .generate_token(admin.id, &admin.username, &admin.role, allowed)?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        admin: profile,
    })))
}

pub async fn me_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<AdminProfile>>> {
    let admin = AdminStore::get_by_id(&state.db, claims.sub).await?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(profile)))
}

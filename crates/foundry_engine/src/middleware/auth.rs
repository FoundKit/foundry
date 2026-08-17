use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use foundry_auth::{AdminClaims, check_system_access};
use foundry_core::context::SystemContext;
use foundry_core::error::AppError;

/// Middleware requiring valid admin JWT token
pub async fn require_admin_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized(
            "Invalid Authorization scheme, expected 'Bearer <token>'".to_string(),
        )
    })?;

    let claims = state.jwt.verify_token(token)?;
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Middleware enforcing topic-scoped RBAC for sub-system administration
pub async fn require_topic_access(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<AdminClaims>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let ctx = req
        .extensions()
        .get::<SystemContext>()
        .ok_or_else(|| AppError::Internal("SystemContext missing".to_string()))?;

    check_system_access(claims, &ctx.system_slug)?;

    Ok(next.run(req).await)
}

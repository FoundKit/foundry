use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::{Method, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use foundry_auth::check_system_access;
use foundry_core::error::AppError;
use serde_json::Value;

/// Middleware enforcing Application or Admin authorization for `/api/s/:system_slug/...` endpoints
pub async fn require_subsystem_app_or_admin_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path().to_string();
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // Support both nested router path (/s/{system_slug}/{sub_resource}/...)
    // and full URI path (/api/s/{system_slug}/{sub_resource}/...)
    let (system_slug, sub_resource) = if segments.len() >= 3 && segments[0] == "s" {
        (segments[1], segments[2])
    } else if segments.len() >= 4 && segments[0] == "api" && segments[1] == "s" {
        (segments[2], segments[3])
    } else {
        return Ok(next.run(req).await);
    };

    let action = match *req.method() {
        Method::GET => "read",
        Method::POST => "create",
        Method::PUT | Method::PATCH => {
            if sub_resource == "configs" {
                "write"
            } else {
                "update"
            }
        }
        Method::DELETE => "delete",
        _ => "read",
    };

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(header) = auth_header {
        let token = header.strip_prefix("Bearer ").unwrap_or(header).trim();

        // 1. Check if token is a valid Admin JWT
        if let Ok(claims) = state.jwt.verify_token(token) {
            check_system_access(&claims, system_slug)?;
            req.extensions_mut().insert(claims);
            return Ok(next.run(req).await);
        }

        // 2. Check if token matches an active client application
        if let Some(app) = state.db.apps().find_by_token(token).await? {
            if app.status != 1 {
                return Err(AppError::Forbidden("Application is disabled".to_string()));
            }

            if app.system_id != system_slug {
                return Err(AppError::Forbidden(
                    "Application does not have access to this subsystem".to_string(),
                ));
            }

            if is_permitted(&app.permissions, sub_resource, action) {
                req.extensions_mut().insert(app);
                return Ok(next.run(req).await);
            }

            return Err(AppError::Forbidden(format!(
                "Application '{}' is not authorized to {} {}",
                app.name, action, sub_resource
            )));
        }

        return Err(AppError::Unauthorized(
            "Invalid or expired authorization token".to_string(),
        ));
    }

    // 3. If no Authorization header is provided:
    // Enforce authorization if this subsystem has active applications registered.
    let has_apps = state.db.apps().has_applications(system_slug).await?;
    if has_apps {
        return Err(AppError::Unauthorized(
            "Missing Authorization header for this subsystem".to_string(),
        ));
    }

    // Backward-compatible fallback for systems without registered applications
    Ok(next.run(req).await)
}

fn is_permitted(permissions: &Value, sub_resource: &str, action: &str) -> bool {
    // Platform-wide root wildcard
    if permissions
        .get("*")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return true;
    }

    if sub_resource == "configs" {
        if let Some(cfg) = permissions.get("configs")
            && (cfg.get(action).and_then(|v| v.as_bool()).unwrap_or(false)
                || cfg.get("*").and_then(|v| v.as_bool()).unwrap_or(false))
        {
            return true;
        }
        return false;
    }

    // Model dynamic CRUD permissions: "models": { "model_slug": { "read": true, "create": true, ... } }
    if let Some(models) = permissions.get("models") {
        // Specific model check
        if let Some(model_cfg) = models.get(sub_resource)
            && check_model_action_perms(model_cfg, action)
        {
            return true;
        }

        // Wildcard model check
        if let Some(all_models_cfg) = models.get("*")
            && check_model_action_perms(all_models_cfg, action)
        {
            return true;
        }
    }

    false
}

fn check_model_action_perms(model_cfg: &Value, action: &str) -> bool {
    match model_cfg {
        Value::Bool(b) => *b,
        Value::Object(map) => {
            if map.get(action).and_then(|v| v.as_bool()).unwrap_or(false) {
                return true;
            }
            if map.get("*").and_then(|v| v.as_bool()).unwrap_or(false) {
                return true;
            }
            false
        }
        Value::Array(arr) => arr.iter().any(|item| {
            item.as_str()
                .map(|s| s == action || s == "*")
                .unwrap_or(false)
        }),
        _ => false,
    }
}

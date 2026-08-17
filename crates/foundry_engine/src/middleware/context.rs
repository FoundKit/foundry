use crate::state::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use foundry_core::context::SystemContext;

/// Middleware to extract tenant SystemContext from path, header, or query
pub async fn extract_system_context(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();

    // Check if the path matches `/api/v1/s/{system_slug}/...` or `/api/v1/admin/s/{system_slug}/...`
    let mut detected_slug = None;

    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments.len() >= 4 && segments[0] == "api" && segments[1] == "v1" && segments[2] == "s" {
        detected_slug = Some(segments[3].to_string());
    } else if segments.len() >= 5
        && segments[0] == "api"
        && segments[1] == "v1"
        && segments[2] == "admin"
        && segments[3] == "s"
    {
        detected_slug = Some(segments[4].to_string());
    }

    // Fallback to X-Foundry-System-ID header
    if detected_slug.is_none()
        && let Some(header_val) = req.headers().get("x-foundry-system-id")
        && let Ok(slug_str) = header_val.to_str()
    {
        detected_slug = Some(slug_str.to_string());
    }

    let locale = req
        .headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .unwrap_or("en-US")
        .to_string();

    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let slug = detected_slug.unwrap_or_else(|| "default".to_string());
    let ctx = SystemContext::with_details(None, &slug, &slug, &locale, client_ip, user_agent);

    req.extensions_mut().insert(ctx);

    next.run(req).await
}

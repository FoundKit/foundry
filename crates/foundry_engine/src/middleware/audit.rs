use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use bytes::Bytes;
use foundry_auth::AdminClaims;
use foundry_core::context::SystemContext;
use foundry_storage::AuditLogInsert;
use http_body_util::BodyExt;
use std::time::Instant;
use tracing::error;

/// Recursively masks sensitive fields in JSON request payloads (e.g. passwords, secrets, tokens).
fn mask_sensitive_json(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_ascii_lowercase();
                if lower == "password"
                    || lower == "old_password"
                    || lower == "new_password"
                    || lower == "confirm_password"
                    || lower == "secret"
                    || lower == "app_secret"
                    || lower == "token"
                    || lower == "code"
                    || lower == "refresh_token"
                    || lower == "access_token"
                    || lower == "api_key"
                    || lower == "private_key"
                {
                    *v = serde_json::json!("[REDACTED]");
                } else {
                    mask_sensitive_json(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                mask_sensitive_json(item);
            }
        }
        _ => {}
    }
}

/// Sanitizes request body bytes for safe storage in audit logs.
pub fn sanitize_body(body_bytes: &[u8]) -> Option<String> {
    if body_bytes.is_empty() {
        return None;
    }

    if let Ok(mut val) = serde_json::from_slice::<serde_json::Value>(body_bytes) {
        mask_sensitive_json(&mut val);
        serde_json::to_string(&val).ok()
    } else {
        String::from_utf8(body_bytes.to_vec()).ok()
    }
}

/// Maps HTTP method and path to a human-readable dynamic action description
pub fn resolve_action_name(method: &Method, path: &str) -> &'static str {
    match (method.as_str(), path) {
        ("POST", p) if p.ends_with("/auth/login") => "管理员登录",
        ("POST", p) if p.ends_with("/auth/email/login") => "邮箱验证码登录",
        ("POST", p) if p.ends_with("/auth/logout") => "管理员登出",
        ("PUT", p) if p.ends_with("/auth/password") => "修改管理员密码",
        ("POST", p) if p.ends_with("/auth/email/send-code") => "发送登录验证码",
        ("POST", p) if p.ends_with("/auth/bind-email/send-code") => "发送绑定邮箱验证码",
        ("POST", p) if p.ends_with("/auth/bind-email/verify") => "完成绑定邮箱",
        ("PUT", "/api/admin/platform/login-settings") => "更新平台登录设置",
        ("POST", "/api/admin/admins") => "新增管理员",
        ("PUT", p) if p.starts_with("/api/admin/admins/") => "修改管理员信息",
        ("POST", "/api/admin/systems") => "创建子系统",
        ("PUT", p) if p.starts_with("/api/admin/systems/") => "修改子系统配置",
        ("POST", p)
            if p.contains("/apps")
                && (p.ends_with("/regenerate-token") || p.ends_with("/regenerate-secret")) =>
        {
            "重置应用密钥"
        }
        ("POST", p) if p.contains("/apps") => "创建客户端应用",
        ("PUT", p) if p.contains("/apps/") => "修改客户端应用",
        ("DELETE", p) if p.contains("/apps/") => "删除客户端应用",
        ("POST", p) if p.contains("/configs/schema") => "配置专题属性项",
        ("PUT", p) if p.contains("/configs") => "修改专题配置值",
        ("POST", p) if p.contains("/models") => "创建数据模型",
        ("POST", p) if p.contains("/fields") => "添加模型字段",
        ("DELETE", p) if p.contains("/records/") || p.contains("/s/") => "删除业务数据记录",
        ("POST", p) if p.contains("/s/") => "新增业务数据记录",
        ("PUT", p) if p.contains("/s/") => "更新业务数据记录",
        ("PATCH", p) if p.contains("/s/") => "部分更新业务数据记录",
        _ => "平台管理写操作",
    }
}

/// Audit middleware intercepting all non-GET requests and login actions
pub async fn audit_interceptor(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query_params = req.uri().query().map(|q| q.to_string());

    // Only audit mutating methods (POST, PUT, PATCH, DELETE) or login paths
    let is_mutating = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    let is_login = path.ends_with("/auth/login") || path.ends_with("/auth/email/login");

    if !is_mutating && !is_login {
        return next.run(req).await;
    }

    let start_time = Instant::now();

    // 1. Try to extract admin or application credentials from request
    let mut admin_claims = req.extensions().get::<AdminClaims>().cloned();
    let mut auth_token_str = None;

    if admin_claims.is_none()
        && let Some(auth_val) = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
    {
        let token = auth_val
            .strip_prefix("Bearer ")
            .or_else(|| auth_val.strip_prefix("bearer "))
            .unwrap_or(auth_val)
            .trim();
        if !token.is_empty() {
            if let Ok(claims) = state.jwt.verify_token(token) {
                admin_claims = Some(claims);
            } else {
                auth_token_str = Some(token.to_string());
            }
        }
    }

    // Extract headers into JSON (redact authorization)
    let mut headers_map = serde_json::Map::new();
    for (k, v) in req.headers() {
        let key = k.as_str();
        if key.eq_ignore_ascii_case("authorization") {
            headers_map.insert(key.to_string(), serde_json::json!("[REDACTED]"));
        } else if let Ok(val) = v.to_str() {
            headers_map.insert(key.to_string(), serde_json::json!(val));
        }
    }
    let headers_json = serde_json::Value::Object(headers_map);

    // Extract body without consuming permanently
    let (parts, body) = req.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    // For login endpoints, attempt to extract the username / email from body
    let mut login_username = None;
    let mut login_email = None;
    if path.ends_with("/auth/login")
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body_bytes)
    {
        login_username = val
            .get("username")
            .and_then(|u| u.as_str())
            .map(ToString::to_string);
    } else if path.ends_with("/auth/email/login")
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body_bytes)
    {
        login_email = val
            .get("email")
            .and_then(|e| e.as_str())
            .map(ToString::to_string);
    }

    // Sanitize body for audit log storage (redact passwords, secrets, tokens)
    let body_params = sanitize_body(&body_bytes);

    // Reconstruct request for downstream handlers
    let mut new_req = Request::from_parts(parts, Body::from(body_bytes));

    if let Some(ref claims) = admin_claims {
        new_req.extensions_mut().insert(claims.clone());
    }

    let system_ctx = new_req.extensions().get::<SystemContext>().cloned();
    let client_ip = system_ctx.as_ref().and_then(|c| c.client_ip.clone());
    let user_agent = system_ctx.as_ref().and_then(|c| c.user_agent.clone());
    let system_slug = system_ctx.map(|c| c.system_slug);

    let response = next.run(new_req).await;
    let duration_ms = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i16;
    let action_name = resolve_action_name(&method, &path);

    let db = state.db.clone();
    let method_str = method.to_string();

    // Asynchronous non-blocking background write
    tokio::spawn(async move {
        // Resolve operator identity
        let (admin_id, admin_username) = if let Some(claims) = admin_claims {
            (Some(claims.sub), Some(claims.username))
        } else if let Some(token) = auth_token_str {
            if let Ok(Some(app)) = db.apps().find_by_token(&token).await {
                (Some(app.id), Some(format!("app:{}", app.name)))
            } else {
                (None, None)
            }
        } else if path.ends_with("/auth/login") {
            if status_code < 400
                && let Some(ref uname) = login_username
            {
                match db.admins().get_by_username(uname).await {
                    Ok(admin) => (Some(admin.id), Some(admin.username)),
                    Err(_) => (None, login_username),
                }
            } else {
                (None, login_username)
            }
        } else if path.ends_with("/auth/email/login") {
            if status_code < 400
                && let Some(ref email) = login_email
            {
                match db.admins().get_by_email(email).await {
                    Ok(admin) => (Some(admin.id), Some(admin.username)),
                    Err(_) => (None, login_email),
                }
            } else {
                (None, login_email)
            }
        } else {
            (None, None)
        };

        let log_insert = AuditLogInsert {
            admin_id,
            admin_username,
            system_slug,
            method: method_str,
            path,
            action_name: Some(action_name.to_string()),
            headers: headers_json,
            query_params,
            body_params,
            ip_address: client_ip,
            user_agent,
            status_code: Some(status_code),
            duration_ms: Some(duration_ms),
        };

        if let Err(e) = db.audit().insert(log_insert).await {
            error!("Failed to persist audit log: {}", e);
        }
    });

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_resolve_action_name() {
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/auth/login"),
            "管理员登录"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/auth/email/login"),
            "邮箱验证码登录"
        );
        assert_eq!(
            resolve_action_name(&Method::PUT, "/api/admin/auth/password"),
            "修改管理员密码"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/admins"),
            "新增管理员"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/systems"),
            "创建子系统"
        );
        assert_eq!(
            resolve_action_name(&Method::PUT, "/api/s/carnival_2026/configs"),
            "修改专题配置值"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/s/carnival_2026/models"),
            "创建数据模型"
        );
        assert_eq!(
            resolve_action_name(&Method::DELETE, "/api/s/carnival_2026/products/1"),
            "删除业务数据记录"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/admin/s/carnival/apps"),
            "创建客户端应用"
        );
        assert_eq!(
            resolve_action_name(
                &Method::POST,
                "/api/admin/s/carnival/apps/1/regenerate-token"
            ),
            "重置应用密钥"
        );
    }

    #[test]
    fn test_sanitize_body_masks_passwords_and_tokens() {
        let raw = serde_json::json!({
            "username": "admin",
            "password": "my_secret_password",
            "nested": {
                "token": "secret_token_123",
                "other": "safe_value"
            }
        });
        let raw_bytes = serde_json::to_vec(&raw).unwrap();
        let sanitized_str = sanitize_body(&raw_bytes).unwrap();
        let sanitized: serde_json::Value = serde_json::from_str(&sanitized_str).unwrap();

        assert_eq!(sanitized["username"], "admin");
        assert_eq!(sanitized["password"], "[REDACTED]");
        assert_eq!(sanitized["nested"]["token"], "[REDACTED]");
        assert_eq!(sanitized["nested"]["other"], "safe_value");
    }

    #[test]
    fn test_sanitize_body_empty() {
        assert_eq!(sanitize_body(&[]), None);
    }

    #[tokio::test]
    async fn test_audit_interceptor_extracts_admin_claims() {
        use axum::routing::post;
        use tower::ServiceExt;

        let db = foundry_storage::Database::dummy();
        let jwt = foundry_auth::JwtService::new("test_secret_1234567890", 24);
        let hooks = foundry_extension::HookPipeline::new();
        let state = AppState::new(db, None, jwt, hooks, vec![]);

        let admin_id = Uuid::new_v4();
        let token = state
            .jwt
            .generate_token(admin_id, "alice", "super_admin", vec![])
            .unwrap();

        let router = axum::Router::new()
            .route(
                "/test-action",
                post(move |req: axum::extract::Request| async move {
                    let claims = req.extensions().get::<AdminClaims>().cloned();
                    assert!(claims.is_some());
                    let claims = claims.unwrap();
                    assert_eq!(claims.username, "alice");
                    assert_eq!(claims.sub, admin_id);
                    axum::http::StatusCode::OK
                }),
            )
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                audit_interceptor,
            ))
            .with_state(state);

        let req = axum::http::Request::builder()
            .uri("/test-action")
            .method("POST")
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(r#"{"action":"test"}"#))
            .unwrap();

        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }
}

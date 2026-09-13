use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use bytes::Bytes;
use foundry_auth::AdminClaims;
use foundry_core::context::SystemContext;
use foundry_storage::{AuditLogInsert, AuditStore};
use http_body_util::BodyExt;
use std::time::Instant;
use tracing::error;

/// Maps HTTP method and path to a human-readable dynamic action description
pub fn resolve_action_name(method: &Method, path: &str) -> &'static str {
    match (method.as_str(), path) {
        ("POST", p) if p.ends_with("/auth/login") => "管理员登录",
        ("POST", p) if p.ends_with("/auth/logout") => "管理员登出",
        ("POST", "/api/v1/admin/admins") => "新增管理员",
        ("PUT", p) if p.starts_with("/api/v1/admin/admins/") => "修改管理员信息",
        ("POST", "/api/v1/admin/systems") => "创建子系统",
        ("PUT", p) if p.starts_with("/api/v1/admin/systems/") => "修改子系统配置",
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
    let is_login = path.ends_with("/auth/login");

    if !is_mutating && !is_login {
        return next.run(req).await;
    }

    let start_time = Instant::now();

    // Extract headers into JSON
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

    let body_str = if body_bytes.is_empty() {
        None
    } else {
        String::from_utf8(body_bytes.to_vec()).ok()
    };

    // Reconstruct request for downstream handlers
    let new_req = Request::from_parts(parts, Body::from(body_bytes));

    let admin_claims = new_req.extensions().get::<AdminClaims>().cloned();
    let system_ctx = new_req.extensions().get::<SystemContext>().cloned();

    let client_ip = system_ctx.as_ref().and_then(|c| c.client_ip.clone());
    let user_agent = system_ctx.as_ref().and_then(|c| c.user_agent.clone());
    let system_slug = system_ctx.map(|c| c.system_slug);

    let response = next.run(new_req).await;
    let duration_ms = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i16;
    let action_name = resolve_action_name(&method, &path);

    let admin_id = admin_claims.as_ref().map(|c| c.sub);
    let admin_username = admin_claims.as_ref().map(|c| c.username.clone());

    let pool = state.db.clone();
    let log_insert = AuditLogInsert {
        admin_id,
        admin_username,
        system_slug,
        method: method.to_string(),
        path,
        action_name: Some(action_name.to_string()),
        headers: headers_json,
        query_params,
        body_params: body_str,
        ip_address: client_ip,
        user_agent,
        status_code: Some(status_code),
        duration_ms: Some(duration_ms),
    };

    // Asynchronous non-blocking background write
    tokio::spawn(async move {
        if let Err(e) = AuditStore::insert(&pool, log_insert).await {
            error!("Failed to persist audit log: {}", e);
        }
    });

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_action_name() {
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/v1/admin/auth/login"),
            "管理员登录"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/v1/admin/admins"),
            "新增管理员"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/v1/admin/systems"),
            "创建子系统"
        );
        assert_eq!(
            resolve_action_name(&Method::PUT, "/api/v1/s/carnival_2026/configs"),
            "修改专题配置值"
        );
        assert_eq!(
            resolve_action_name(&Method::POST, "/api/v1/admin/s/carnival_2026/models"),
            "创建数据模型"
        );
        assert_eq!(
            resolve_action_name(&Method::DELETE, "/api/v1/s/carnival_2026/products/1"),
            "删除业务数据记录"
        );
    }
}

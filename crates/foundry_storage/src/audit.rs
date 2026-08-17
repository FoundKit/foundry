use crate::db::DbPool;
use crate::entities::AuditLogEntity;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogInsert {
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub system_slug: Option<String>,
    pub method: String,
    pub path: String,
    pub action_name: Option<String>,
    pub headers: serde_json::Value,
    pub query_params: Option<String>,
    pub body_params: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status_code: Option<i16>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuditLogQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub admin_id: Option<Uuid>,
    pub system_slug: Option<String>,
    pub method: Option<String>,
}

pub struct AuditStore;

impl AuditStore {
    pub async fn insert(pool: &DbPool, log: AuditLogInsert) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                admin_id, admin_username, system_slug, method, path, action_name,
                headers, query_params, body_params, ip_address, user_agent, status_code, duration_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(log.admin_id)
        .bind(log.admin_username)
        .bind(log.system_slug)
        .bind(log.method)
        .bind(log.path)
        .bind(log.action_name)
        .bind(log.headers)
        .bind(log.query_params)
        .bind(log.body_params)
        .bind(log.ip_address)
        .bind(log.user_agent)
        .bind(log.status_code)
        .bind(log.duration_ms)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to insert audit log: {}", e)))?;

        Ok(())
    }

    pub async fn list(
        pool: &DbPool,
        query: AuditLogQuery,
    ) -> AppResult<PaginatedData<AuditLogEntity>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let total_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_logs
            WHERE ($1::uuid IS NULL OR admin_id = $1)
              AND ($2::varchar IS NULL OR system_slug = $2)
              AND ($3::varchar IS NULL OR method = $3)
            "#,
        )
        .bind(query.admin_id)
        .bind(query.system_slug.as_deref())
        .bind(query.method.as_deref())
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Audit count failed: {}", e)))?;

        let total = total_row.0 as u64;

        let logs = sqlx::query_as::<_, AuditLogEntity>(
            r#"
            SELECT id, admin_id, admin_username, system_slug, method, path, action_name,
                   headers, query_params, body_params, ip_address, user_agent, status_code, duration_ms, created_at
            FROM audit_logs
            WHERE ($1::uuid IS NULL OR admin_id = $1)
              AND ($2::varchar IS NULL OR system_slug = $2)
              AND ($3::varchar IS NULL OR method = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(query.admin_id)
        .bind(query.system_slug.as_deref())
        .bind(query.method.as_deref())
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Audit fetch failed: {}", e)))?;

        Ok(PaginatedData::new(logs, page, page_size, total))
    }
}

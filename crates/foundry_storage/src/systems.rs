use crate::db::DbPool;
use crate::entities::{PlatformSummary, SystemEntity, SystemItem, SystemStats};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SystemQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub id: Option<Uuid>,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i16>,
}

pub struct SystemStore;

impl SystemStore {
    /// Retrieve list of all sub-systems (legacy / simple)
    pub async fn list(pool: &DbPool) -> AppResult<Vec<SystemEntity>> {
        let rows = sqlx::query_as::<_, SystemEntity>(
            r#"
            SELECT id, slug, name, description, status, created_at, updated_at, deleted_at
            FROM systems
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list systems: {}", e)))?;

        Ok(rows)
    }

    /// Retrieve paginated list of sub-systems with multi-attribute filtering & live statistics
    pub async fn list_paginated(
        pool: &DbPool,
        query: SystemQuery,
        allowed_systems: Option<&[String]>,
    ) -> AppResult<PaginatedData<SystemItem>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(10).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let slug_filter = query
            .slug
            .as_ref()
            .map(|s| format!("%{}%", s.trim().to_lowercase()));
        let name_filter = query.name.as_ref().map(|n| format!("%{}%", n.trim()));
        let kw_filter = query
            .keyword
            .as_ref()
            .map(|k| format!("%{}%", k.trim().to_lowercase()));

        let has_allowed_check = allowed_systems.is_some();
        let allowed_list = allowed_systems.unwrap_or(&[]);

        // 1. Total count query
        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM systems
            WHERE deleted_at IS NULL
              AND ($1::bool = false OR slug = ANY($2))
              AND ($3::uuid IS NULL OR id = $3)
              AND ($4::varchar IS NULL OR LOWER(slug) LIKE $4)
              AND ($5::varchar IS NULL OR name ILIKE $5)
              AND ($6::varchar IS NULL OR (LOWER(slug) LIKE $6 OR name ILIKE $6 OR COALESCE(description, '') ILIKE $6))
              AND ($7::smallint IS NULL OR status = $7)
            "#,
        )
        .bind(has_allowed_check)
        .bind(allowed_list)
        .bind(query.id)
        .bind(slug_filter.as_deref())
        .bind(name_filter.as_deref())
        .bind(kw_filter.as_deref())
        .bind(query.status)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count systems: {}", e)))?;

        let total = count_row.0 as u64;

        // 2. Fetch paginated records enriched with models_count, configs_count, records_count
        let rows = sqlx::query_as::<_, SystemItem>(
            r#"
            SELECT
                s.id,
                s.slug,
                s.name,
                s.description,
                s.status,
                s.created_at,
                s.updated_at,
                s.deleted_at,
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0)::bigint AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0)::bigint AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0)::bigint AS records_count
            FROM systems s
            WHERE s.deleted_at IS NULL
              AND ($1::bool = false OR s.slug = ANY($2))
              AND ($3::uuid IS NULL OR s.id = $3)
              AND ($4::varchar IS NULL OR LOWER(s.slug) LIKE $4)
              AND ($5::varchar IS NULL OR s.name ILIKE $5)
              AND ($6::varchar IS NULL OR (LOWER(s.slug) LIKE $6 OR s.name ILIKE $6 OR COALESCE(s.description, '') ILIKE $6))
              AND ($7::smallint IS NULL OR s.status = $7)
            ORDER BY s.created_at ASC
            LIMIT $8 OFFSET $9
            "#,
        )
        .bind(has_allowed_check)
        .bind(allowed_list)
        .bind(query.id)
        .bind(slug_filter.as_deref())
        .bind(name_filter.as_deref())
        .bind(kw_filter.as_deref())
        .bind(query.status)
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch systems: {}", e)))?;

        Ok(PaginatedData::new(rows, page, page_size, total))
    }

    /// Retrieve single sub-system by UUID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> AppResult<SystemItem> {
        let row = sqlx::query_as::<_, SystemItem>(
            r#"
            SELECT
                s.id,
                s.slug,
                s.name,
                s.description,
                s.status,
                s.created_at,
                s.updated_at,
                s.deleted_at,
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0)::bigint AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0)::bigint AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0)::bigint AS records_count
            FROM systems s
            WHERE s.id = $1 AND s.deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        Ok(row)
    }

    /// Retrieve single sub-system by unique slug
    pub async fn get_by_slug(pool: &DbPool, slug: &str) -> AppResult<SystemItem> {
        let row = sqlx::query_as::<_, SystemItem>(
            r#"
            SELECT
                s.id,
                s.slug,
                s.name,
                s.description,
                s.status,
                s.created_at,
                s.updated_at,
                s.deleted_at,
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0)::bigint AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0)::bigint AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0)::bigint AS records_count
            FROM systems s
            WHERE s.slug = $1 AND s.deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system '{}' not found", slug)))?;

        Ok(row)
    }

    /// Retrieve detailed stats for a sub-system
    pub async fn get_stats(pool: &DbPool, slug: &str) -> AppResult<SystemStats> {
        let sys = Self::get_by_slug(pool, slug).await?;

        let audit_count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_logs
            WHERE system_slug = $1
            "#,
        )
        .bind(slug)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count audit logs: {}", e)))?;

        Ok(SystemStats {
            id: sys.id,
            slug: sys.slug,
            name: sys.name,
            description: sys.description,
            status: sys.status,
            created_at: sys.created_at,
            models_count: sys.models_count as u64,
            configs_count: sys.configs_count as u64,
            records_count: sys.records_count as u64,
            audit_logs_count: audit_count_row.0 as u64,
        })
    }

    /// Retrieve platform-wide summary metrics
    pub async fn platform_summary(pool: &DbPool) -> AppResult<PlatformSummary> {
        let (total_systems, active_systems): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint AS total,
                COUNT(CASE WHEN status = 1 THEN 1 END)::bigint AS active
            FROM systems
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query system stats: {}", e)))?;

        let total_models: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM models WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count models: {}", e)))?;

        let total_records: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM model_records WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count records: {}", e)))?;

        let total_admins: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM admins
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count admins: {}", e)))?;

        let total_audit_logs: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM audit_logs
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count audit logs: {}", e)))?;

        Ok(PlatformSummary {
            total_systems: total_systems as u64,
            active_systems: active_systems as u64,
            total_models: total_models.0 as u64,
            total_records: total_records.0 as u64,
            total_admins: total_admins.0 as u64,
            total_audit_logs: total_audit_logs.0 as u64,
        })
    }

    /// Create a new sub-system
    pub async fn create(
        pool: &DbPool,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<SystemEntity> {
        let row = sqlx::query_as::<_, SystemEntity>(
            r#"
            INSERT INTO systems (slug, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, slug, name, description, status, created_at, updated_at, deleted_at
            "#,
        )
        .bind(slug)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create system: {}", e)))?;

        Ok(row)
    }

    /// Update an existing sub-system
    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> AppResult<SystemEntity> {
        let row = sqlx::query_as::<_, SystemEntity>(
            r#"
            UPDATE systems
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, slug, name, description, status, created_at, updated_at, deleted_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(status)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        Ok(row)
    }
}

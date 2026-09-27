use crate::entities::{
    AdminEntity, AuditLogEntity, ModelEntity, ModelFieldEntity, PlatformSummary,
    SystemConfigEntity, SystemEntity, SystemItem, SystemStats,
};
use crate::traits::{
    AdminStoreEngine, AuditLogInsert, AuditLogQuery, AuditStoreEngine, ConfigStoreEngine,
    ModelStoreEngine, SystemQuery, SystemStoreEngine,
};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// PostgresModelEngine
// ============================================================================

pub struct PostgresModelEngine {
    pool: PgPool,
}

impl PostgresModelEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelStoreEngine for PostgresModelEngine {
    async fn list_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        let rows = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE system_id = $1 AND deleted_at IS NULL
            ORDER BY id ASC
            "#,
        )
        .bind(system_slug)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list models: {}", e)))?;

        Ok(rows)
    }

    async fn get_model(&self, system_slug: &str, model_slug: &str) -> AppResult<ModelEntity> {
        let row = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE system_id = $1 AND slug = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get model: {}", e)))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Model '{}' not found in system '{}'",
                model_slug, system_slug
            ))
        })?;

        Ok(row)
    }

    async fn create_model(
        &self,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        let default_perms = serde_json::json!({
            "public_read": false,
            "public_write": false,
            "auth_read": true,
            "auth_write": false
        });
        let perms = permissions.unwrap_or(default_perms);

        let row = sqlx::query_as::<_, ModelEntity>(
            r#"
            INSERT INTO models (system_id, slug, name, description, permissions)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            "#,
        )
        .bind(system_slug)
        .bind(slug)
        .bind(name)
        .bind(description)
        .bind(perms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create model: {}", e)))?;

        Ok(row)
    }

    async fn list_fields(&self, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        let rows = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            SELECT id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            FROM model_fields
            WHERE model_id = $1
            ORDER BY sort_order ASC, id ASC
            "#,
        )
        .bind(model_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list fields: {}", e)))?;

        Ok(rows)
    }

    async fn add_field(
        &self,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        let opts = options.unwrap_or_else(|| serde_json::json!({}));
        let row = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            INSERT INTO model_fields (model_id, name, label, field_type, is_required, default_value, options, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            "#,
        )
        .bind(model_id)
        .bind(name)
        .bind(label)
        .bind(field_type)
        .bind(is_required)
        .bind(default_value)
        .bind(opts)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to add model field: {}", e)))?;

        Ok(row)
    }
}

// ============================================================================
// PostgresSystemEngine
// ============================================================================

pub struct PostgresSystemEngine {
    pool: PgPool,
}

impl PostgresSystemEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemStoreEngine for PostgresSystemEngine {
    async fn list(&self) -> AppResult<Vec<SystemEntity>> {
        let rows = sqlx::query_as::<_, SystemEntity>(
            r#"
            SELECT id, slug, name, description, status, created_at, updated_at, deleted_at
            FROM systems
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list systems: {}", e)))?;

        Ok(rows)
    }

    async fn list_paginated(
        &self,
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count systems: {}", e)))?;

        let total = count_row.0 as u64;

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
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch systems: {}", e)))?;

        Ok(PaginatedData::new(rows, page, page_size, total))
    }

    async fn get_by_id(&self, id: Uuid) -> AppResult<SystemItem> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        Ok(row)
    }

    async fn get_by_slug(&self, slug: &str) -> AppResult<SystemItem> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system '{}' not found", slug)))?;

        Ok(row)
    }

    async fn get_stats(&self, slug: &str) -> AppResult<SystemStats> {
        let sys = self.get_by_slug(slug).await?;

        let audit_count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_logs
            WHERE system_slug = $1
            "#,
        )
        .bind(slug)
        .fetch_one(&self.pool)
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

    async fn platform_summary(&self) -> AppResult<PlatformSummary> {
        let (total_systems, active_systems): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint AS total,
                COUNT(CASE WHEN status = 1 THEN 1 END)::bigint AS active
            FROM systems
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query system stats: {}", e)))?;

        let total_models: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM models WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count models: {}", e)))?;

        let total_records: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM model_records WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count records: {}", e)))?;

        let total_admins: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM admins
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count admins: {}", e)))?;

        let total_audit_logs: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint FROM audit_logs
            "#,
        )
        .fetch_one(&self.pool)
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

    async fn create(
        &self,
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create system: {}", e)))?;

        Ok(row)
    }

    async fn update(
        &self,
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
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        Ok(row)
    }
}

// ============================================================================
// PostgresConfigEngine
// ============================================================================

pub struct PostgresConfigEngine {
    pool: PgPool,
}

impl PostgresConfigEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConfigStoreEngine for PostgresConfigEngine {
    async fn list(&self, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>> {
        let rows = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            SELECT id, system_id, key, label, value_type, value, options, sort_order, created_at, updated_at
            FROM system_configs
            WHERE system_id = $1
            ORDER BY sort_order ASC, id ASC
            "#,
        )
        .bind(system_slug)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch configs: {}", e)))?;

        Ok(rows)
    }

    async fn get_aggregated(&self, system_slug: &str) -> AppResult<Value> {
        let configs = self.list(system_slug).await?;
        let mut map = serde_json::Map::new();

        for cfg in configs {
            let val = cfg.value.unwrap_or(Value::Null);
            map.insert(cfg.key, val);
        }

        Ok(Value::Object(map))
    }

    async fn upsert(
        &self,
        system_slug: &str,
        key: &str,
        label: &str,
        value_type: &str,
        value: Option<Value>,
        options: Value,
        sort_order: i32,
    ) -> AppResult<SystemConfigEntity> {
        let row = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            INSERT INTO system_configs (system_id, key, label, value_type, value, options, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (system_id, key)
            DO UPDATE SET
                label = EXCLUDED.label,
                value_type = EXCLUDED.value_type,
                value = EXCLUDED.value,
                options = EXCLUDED.options,
                sort_order = EXCLUDED.sort_order,
                updated_at = NOW()
            RETURNING id, system_id, key, label, value_type, value, options, sort_order, created_at, updated_at
            "#,
        )
        .bind(system_slug)
        .bind(key)
        .bind(label)
        .bind(value_type)
        .bind(value)
        .bind(options)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to upsert config: {}", e)))?;

        Ok(row)
    }

    async fn update_values(
        &self,
        system_slug: &str,
        values: &serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (k, v) in values {
            sqlx::query(
                r#"
                UPDATE system_configs
                SET value = $1, updated_at = NOW()
                WHERE system_id = $2 AND key = $3
                "#,
            )
            .bind(v)
            .bind(system_slug)
            .bind(k)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Database(format!("Failed to update config {}: {}", k, e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// PostgresAdminEngine
// ============================================================================

pub struct PostgresAdminEngine {
    pool: PgPool,
}

impl PostgresAdminEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminStoreEngine for PostgresAdminEngine {
    async fn list(&self) -> AppResult<Vec<AdminEntity>> {
        let rows = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list admins: {}", e)))?;

        Ok(rows)
    }

    async fn get_by_id(&self, id: Uuid) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get admin: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin #{} not found", id)))?;

        Ok(row)
    }

    async fn get_by_username(&self, username: &str) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get admin by username: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin '{}' not found", username)))?;

        Ok(row)
    }

    async fn create(
        &self,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: &str,
        allowed_systems: Value,
    ) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            INSERT INTO admins (username, email, password_hash, role, allowed_systems)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create admin: {}", e)))?;

        Ok(row)
    }

    async fn update(
        &self,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<Value>,
        status: Option<i16>,
    ) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            UPDATE admins
            SET
                email = COALESCE($2, email),
                password_hash = COALESCE($3, password_hash),
                role = COALESCE($4, role),
                allowed_systems = COALESCE($5, allowed_systems),
                status = COALESCE($6, status),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .bind(status)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update admin: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin #{} not found", id)))?;

        Ok(row)
    }
}

// ============================================================================
// PostgresAuditEngine
// ============================================================================

pub struct PostgresAuditEngine {
    pool: PgPool,
}

impl PostgresAuditEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditStoreEngine for PostgresAuditEngine {
    async fn insert(&self, log: AuditLogInsert) -> AppResult<()> {
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
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to insert audit log: {}", e)))?;

        Ok(())
    }

    async fn list(&self, query: AuditLogQuery) -> AppResult<PaginatedData<AuditLogEntity>> {
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
        .fetch_one(&self.pool)
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
            "#,
        )
        .bind(query.admin_id)
        .bind(query.system_slug.as_deref())
        .bind(query.method.as_deref())
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Audit fetch failed: {}", e)))?;

        Ok(PaginatedData::new(logs, page, page_size, total))
    }
}

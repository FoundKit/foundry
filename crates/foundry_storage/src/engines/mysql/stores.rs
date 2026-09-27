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
use sqlx::{MySql, MySqlPool, QueryBuilder};
use uuid::Uuid;

// ============================================================================
// MySqlModelEngine
// ============================================================================

pub struct MySqlModelEngine {
    pool: MySqlPool,
}

impl MySqlModelEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelStoreEngine for MySqlModelEngine {
    async fn list_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        let rows = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE system_id = ? AND deleted_at IS NULL
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
            WHERE system_id = ? AND slug = ? AND deleted_at IS NULL
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

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            INSERT INTO models (system_id, slug, name, description, permissions)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(system_slug)
        .bind(slug)
        .bind(name)
        .bind(description)
        .bind(&perms)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create model: {}", e)))?;

        let id = res.last_insert_id() as i64;

        let row = sqlx::query_as::<_, ModelEntity>(
            r#"
            SELECT id, system_id, slug, name, description, is_system, status, permissions, created_at, updated_at, deleted_at
            FROM models
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch created model: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(row)
    }

    async fn list_fields(&self, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        let rows = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            SELECT id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            FROM model_fields
            WHERE model_id = ?
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

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            INSERT INTO model_fields (model_id, name, label, field_type, is_required, default_value, options, sort_order)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(model_id)
        .bind(name)
        .bind(label)
        .bind(field_type)
        .bind(is_required)
        .bind(&default_value)
        .bind(&opts)
        .bind(sort_order)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to add model field: {}", e)))?;

        let id = res.last_insert_id() as i64;

        let row = sqlx::query_as::<_, ModelFieldEntity>(
            r#"
            SELECT id, model_id, name, label, field_type, is_required, default_value, options, sort_order, created_at, updated_at
            FROM model_fields
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch added field: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(row)
    }
}

// ============================================================================
// MySqlSystemEngine
// ============================================================================

pub struct MySqlSystemEngine {
    pool: MySqlPool,
}

impl MySqlSystemEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemStoreEngine for MySqlSystemEngine {
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

        // 1. Build Count Query
        let mut count_builder =
            QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM systems WHERE deleted_at IS NULL");

        if let Some(allowed) = allowed_systems {
            count_builder.push(" AND slug IN (");
            let mut sep = count_builder.separated(", ");
            for s in allowed {
                sep.push_bind(s);
            }
            if allowed.is_empty() {
                sep.push_bind("___EMPTY_ALLOWED_GUARD___");
            }
            count_builder.push(")");
        }

        if let Some(ref id) = query.id {
            count_builder.push(" AND id = ");
            count_builder.push_bind(*id);
        }

        if let Some(ref slug) = query.slug {
            count_builder.push(" AND LOWER(slug) LIKE ");
            count_builder.push_bind(format!("%{}%", slug.trim().to_lowercase()));
        }

        if let Some(ref name) = query.name {
            count_builder.push(" AND name LIKE ");
            count_builder.push_bind(format!("%{}%", name.trim()));
        }

        if let Some(ref kw) = query.keyword {
            let kw_pattern = format!("%{}%", kw.trim().to_lowercase());
            count_builder.push(" AND (LOWER(slug) LIKE ");
            count_builder.push_bind(kw_pattern.clone());
            count_builder.push(" OR LOWER(name) LIKE ");
            count_builder.push_bind(kw_pattern.clone());
            count_builder.push(" OR LOWER(COALESCE(description, '')) LIKE ");
            count_builder.push_bind(kw_pattern);
            count_builder.push(")");
        }

        if let Some(status) = query.status {
            count_builder.push(" AND status = ");
            count_builder.push_bind(status);
        }

        let total_row: (i64,) = count_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to count systems: {}", e)))?;

        let total = total_row.0 as u64;

        // 2. Build Paginated Query with subquery counts
        let mut data_builder = QueryBuilder::<MySql>::new(
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
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0) AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0) AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0) AS records_count
            FROM systems s
            WHERE s.deleted_at IS NULL
            "#,
        );

        if let Some(allowed) = allowed_systems {
            data_builder.push(" AND s.slug IN (");
            let mut sep = data_builder.separated(", ");
            for s in allowed {
                sep.push_bind(s);
            }
            if allowed.is_empty() {
                sep.push_bind("___EMPTY_ALLOWED_GUARD___");
            }
            data_builder.push(")");
        }

        if let Some(ref id) = query.id {
            data_builder.push(" AND s.id = ");
            data_builder.push_bind(*id);
        }

        if let Some(ref slug) = query.slug {
            data_builder.push(" AND LOWER(s.slug) LIKE ");
            data_builder.push_bind(format!("%{}%", slug.trim().to_lowercase()));
        }

        if let Some(ref name) = query.name {
            data_builder.push(" AND s.name LIKE ");
            data_builder.push_bind(format!("%{}%", name.trim()));
        }

        if let Some(ref kw) = query.keyword {
            let kw_pattern = format!("%{}%", kw.trim().to_lowercase());
            data_builder.push(" AND (LOWER(s.slug) LIKE ");
            data_builder.push_bind(kw_pattern.clone());
            data_builder.push(" OR LOWER(s.name) LIKE ");
            data_builder.push_bind(kw_pattern.clone());
            data_builder.push(" OR LOWER(COALESCE(s.description, '')) LIKE ");
            data_builder.push_bind(kw_pattern);
            data_builder.push(")");
        }

        if let Some(status) = query.status {
            data_builder.push(" AND s.status = ");
            data_builder.push_bind(status);
        }

        data_builder.push(" ORDER BY s.created_at ASC LIMIT ");
        data_builder.push_bind(page_size);
        data_builder.push(" OFFSET ");
        data_builder.push_bind(offset);

        let rows = data_builder
            .build_query_as::<SystemItem>()
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
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0) AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0) AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0) AS records_count
            FROM systems s
            WHERE s.id = ? AND s.deleted_at IS NULL
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
                COALESCE((SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug AND m.deleted_at IS NULL), 0) AS models_count,
                COALESCE((SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug), 0) AS configs_count,
                COALESCE((SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug AND r.deleted_at IS NULL), 0) AS records_count
            FROM systems s
            WHERE s.slug = ? AND s.deleted_at IS NULL
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
            WHERE system_slug = ?
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
                COUNT(*) AS total,
                COUNT(CASE WHEN status = 1 THEN 1 END) AS active
            FROM systems
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query system stats: {}", e)))?;

        let total_models: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM models WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count models: {}", e)))?;

        let total_records: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM model_records WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count records: {}", e)))?;

        let total_admins: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM admins
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count admins: {}", e)))?;

        let total_audit_logs: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM audit_logs
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
        let id = Uuid::new_v4();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO systems (id, slug, name, description)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(slug)
        .bind(name)
        .bind(description)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create system: {}", e)))?;

        let row = sqlx::query_as::<_, SystemEntity>(
            r#"
            SELECT id, slug, name, description, status, created_at, updated_at, deleted_at
            FROM systems
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch created system: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(row)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> AppResult<SystemEntity> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            UPDATE systems
            SET
                name = COALESCE(?, name),
                description = COALESCE(?, description),
                status = COALESCE(?, status),
                updated_at = NOW(6)
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(status)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update system: {}", e)))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Sub-system #{} not found", id)));
        }

        let row = sqlx::query_as::<_, SystemEntity>(
            r#"
            SELECT id, slug, name, description, status, created_at, updated_at, deleted_at
            FROM systems
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch updated system: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(row)
    }
}

// ============================================================================
// MySqlConfigEngine
// ============================================================================

pub struct MySqlConfigEngine {
    pool: MySqlPool,
}

impl MySqlConfigEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConfigStoreEngine for MySqlConfigEngine {
    async fn list(&self, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>> {
        let rows = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            SELECT id, system_id, `key`, label, value_type, value, options, sort_order, created_at, updated_at
            FROM system_configs
            WHERE system_id = ?
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO system_configs (system_id, `key`, label, value_type, value, options, sort_order)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                label = VALUES(label),
                value_type = VALUES(value_type),
                value = VALUES(value),
                options = VALUES(options),
                sort_order = VALUES(sort_order),
                updated_at = NOW(6)
            "#,
        )
        .bind(system_slug)
        .bind(key)
        .bind(label)
        .bind(value_type)
        .bind(&value)
        .bind(&options)
        .bind(sort_order)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to upsert config: {}", e)))?;

        let row = sqlx::query_as::<_, SystemConfigEntity>(
            r#"
            SELECT id, system_id, `key`, label, value_type, value, options, sort_order, created_at, updated_at
            FROM system_configs
            WHERE system_id = ? AND `key` = ?
            "#,
        )
        .bind(system_slug)
        .bind(key)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch upserted config: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

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
                SET value = ?, updated_at = NOW(6)
                WHERE system_id = ? AND `key` = ?
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
// MySqlAdminEngine
// ============================================================================

pub struct MySqlAdminEngine {
    pool: MySqlPool,
}

impl MySqlAdminEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminStoreEngine for MySqlAdminEngine {
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
            WHERE id = ?
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
            WHERE username = ?
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
        let id = Uuid::new_v4();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO admins (id, username, email, password_hash, role, allowed_systems)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(&allowed_systems)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create admin: {}", e)))?;

        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch created admin: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            UPDATE admins
            SET
                email = COALESCE(?, email),
                password_hash = COALESCE(?, password_hash),
                role = COALESCE(?, role),
                allowed_systems = COALESCE(?, allowed_systems),
                status = COALESCE(?, status),
                updated_at = NOW(6)
            WHERE id = ?
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(&allowed_systems)
        .bind(status)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update admin: {}", e)))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Admin #{} not found", id)));
        }

        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch updated admin: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(row)
    }
}

// ============================================================================
// MySqlAuditEngine
// ============================================================================

pub struct MySqlAuditEngine {
    pool: MySqlPool,
}

impl MySqlAuditEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditStoreEngine for MySqlAuditEngine {
    async fn insert(&self, log: AuditLogInsert) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                admin_id, admin_username, system_slug, method, path, action_name,
                headers, query_params, body_params, ip_address, user_agent, status_code, duration_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(log.admin_id)
        .bind(log.admin_username)
        .bind(log.system_slug)
        .bind(log.method)
        .bind(log.path)
        .bind(log.action_name)
        .bind(&log.headers)
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

        let mut count_builder =
            QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM audit_logs WHERE 1=1");
        if let Some(admin_id) = query.admin_id {
            count_builder.push(" AND admin_id = ");
            count_builder.push_bind(admin_id);
        }
        if let Some(ref slug) = query.system_slug {
            count_builder.push(" AND system_slug = ");
            count_builder.push_bind(slug);
        }
        if let Some(ref method) = query.method {
            count_builder.push(" AND method = ");
            count_builder.push_bind(method);
        }

        let total_row: (i64,) = count_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Audit count failed: {}", e)))?;

        let total = total_row.0 as u64;

        let mut data_builder = QueryBuilder::<MySql>::new(
            r#"
            SELECT id, admin_id, admin_username, system_slug, method, path, action_name,
                   headers, query_params, body_params, ip_address, user_agent, status_code, duration_ms, created_at
            FROM audit_logs
            WHERE 1=1
            "#,
        );
        if let Some(admin_id) = query.admin_id {
            data_builder.push(" AND admin_id = ");
            data_builder.push_bind(admin_id);
        }
        if let Some(ref slug) = query.system_slug {
            data_builder.push(" AND system_slug = ");
            data_builder.push_bind(slug);
        }
        if let Some(ref method) = query.method {
            data_builder.push(" AND method = ");
            data_builder.push_bind(method);
        }

        data_builder.push(" ORDER BY created_at DESC LIMIT ");
        data_builder.push_bind(page_size);
        data_builder.push(" OFFSET ");
        data_builder.push_bind(offset);

        let logs = data_builder
            .build_query_as::<AuditLogEntity>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Audit fetch failed: {}", e)))?;

        Ok(PaginatedData::new(logs, page, page_size, total))
    }
}

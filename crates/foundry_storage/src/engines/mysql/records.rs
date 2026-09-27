use crate::entities::ModelRecordEntity;
use crate::traits::{RecordQuery, RecordStoreEngine};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use serde_json::Value;
use sqlx::MySqlPool;

pub struct MySqlRecordEngine {
    pool: MySqlPool,
}

impl MySqlRecordEngine {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    async fn resolve_model_id(&self, system_slug: &str, model_slug: &str) -> AppResult<i64> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT id FROM models
            WHERE system_id = ? AND slug = ? AND deleted_at IS NULL
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to resolve model ID: {}", e)))?;

        row.map(|r| r.0).ok_or_else(|| {
            AppError::NotFound(format!(
                "Model '{}' not found in system '{}'",
                model_slug, system_slug
            ))
        })
    }
}

#[async_trait]
impl RecordStoreEngine for MySqlRecordEngine {
    async fn list(
        &self,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>> {
        let model_id = self.resolve_model_id(system_slug, model_slug).await?;

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // 1. Total count query on covering index
        let total_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM model_records
            WHERE model_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(model_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Count query failed: {}", e)))?;

        let total = total_row.0 as u64;

        // 2. Deferred Join Optimization:
        // Subquery scans only the lightweight covering index `idx_model_query` to get IDs,
        // then joins back to fetch the heavy JSON data column.
        let records = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT r.id, r.system_id, r.model_slug, r.data, r.created_at, r.updated_at, r.deleted_at
            FROM (
                SELECT id FROM model_records
                WHERE model_id = ? AND deleted_at IS NULL
                ORDER BY created_at DESC, id DESC
                LIMIT ? OFFSET ?
            ) AS p
            JOIN model_records r ON r.id = p.id
            ORDER BY r.created_at DESC, r.id DESC
            "#,
        )
        .bind(model_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Fetch records failed: {}", e)))?;

        Ok(PaginatedData::new(records, page, page_size, total))
    }

    async fn get_by_id(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<ModelRecordEntity> {
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE id = ? AND system_id = ? AND model_slug = ? AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Get record failed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        Ok(record)
    }

    async fn create(
        &self,
        system_slug: &str,
        model_slug: &str,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let model_id = self.resolve_model_id(system_slug, model_slug).await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            INSERT INTO model_records (model_id, system_id, model_slug, data)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(model_id)
        .bind(system_slug)
        .bind(model_slug)
        .bind(&data)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Create record failed: {}", e)))?;

        let inserted_id = res.last_insert_id() as i64;

        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE id = ?
            "#,
        )
        .bind(inserted_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch created record: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(record)
    }

    async fn update(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let res = sqlx::query(
            r#"
            UPDATE model_records
            SET data = ?, updated_at = NOW(6)
            WHERE id = ? AND system_id = ? AND model_slug = ? AND deleted_at IS NULL
            "#,
        )
        .bind(&data)
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Update record failed: {}", e)))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch updated record: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

        Ok(record)
    }

    async fn delete(&self, system_slug: &str, model_slug: &str, id: i64) -> AppResult<()> {
        let res = sqlx::query(
            r#"
            UPDATE model_records
            SET deleted_at = NOW(6)
            WHERE id = ? AND system_id = ? AND model_slug = ? AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Delete record failed: {}", e)))?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        Ok(())
    }
}

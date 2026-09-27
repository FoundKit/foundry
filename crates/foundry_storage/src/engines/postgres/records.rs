use crate::entities::ModelRecordEntity;
use crate::traits::{RecordQuery, RecordStoreEngine};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use serde_json::Value;
use sqlx::PgPool;

pub struct PostgresRecordEngine {
    pool: PgPool,
}

impl PostgresRecordEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecordStoreEngine for PostgresRecordEngine {
    async fn list(
        &self,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let total_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM model_records
            WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Count query failed: {}", e)))?;

        let total = total_row.0 as u64;

        let records = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            SELECT id, system_id, model_slug, data, created_at, updated_at, deleted_at
            FROM model_records
            WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .bind(page_size as i64)
        .bind(offset as i64)
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
            WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL
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
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            INSERT INTO model_records (system_id, model_slug, data)
            VALUES ($1, $2, $3)
            RETURNING id, system_id, model_slug, data, created_at, updated_at, deleted_at
            "#,
        )
        .bind(system_slug)
        .bind(model_slug)
        .bind(data)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Create record failed: {}", e)))?;

        Ok(record)
    }

    async fn update(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let record = sqlx::query_as::<_, ModelRecordEntity>(
            r#"
            UPDATE model_records
            SET data = $1, updated_at = NOW()
            WHERE id = $2 AND system_id = $3 AND model_slug = $4 AND deleted_at IS NULL
            RETURNING id, system_id, model_slug, data, created_at, updated_at, deleted_at
            "#,
        )
        .bind(data)
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Update record failed: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        Ok(record)
    }

    async fn delete(&self, system_slug: &str, model_slug: &str, id: i64) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE model_records
            SET deleted_at = NOW()
            WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(system_slug)
        .bind(model_slug)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Delete record failed: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        Ok(())
    }
}

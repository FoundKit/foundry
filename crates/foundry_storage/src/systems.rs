use crate::db::DbPool;
use crate::entities::SystemEntity;
use foundry_core::error::{AppError, AppResult};
use uuid::Uuid;

pub struct SystemStore;

impl SystemStore {
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

    pub async fn get_by_slug(pool: &DbPool, slug: &str) -> AppResult<SystemEntity> {
        let row = sqlx::query_as::<_, SystemEntity>(
            r#"
            SELECT id, slug, name, description, status, created_at, updated_at, deleted_at
            FROM systems
            WHERE slug = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get system: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Sub-system '{}' not found", slug)))?;

        Ok(row)
    }

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

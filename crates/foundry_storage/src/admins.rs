use crate::db::DbPool;
use crate::entities::AdminEntity;
use foundry_core::error::{AppError, AppResult};
use uuid::Uuid;

pub struct AdminStore;

impl AdminStore {
    pub async fn list(pool: &DbPool) -> AppResult<Vec<AdminEntity>> {
        let rows = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list admins: {}", e)))?;

        Ok(rows)
    }

    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get admin: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin #{} not found", id)))?;

        Ok(row)
    }

    pub async fn get_by_username(pool: &DbPool, username: &str) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            SELECT id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            FROM admins
            WHERE username = $1
            "#
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get admin by username: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin '{}' not found", username)))?;

        Ok(row)
    }

    pub async fn create(
        pool: &DbPool,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: &str,
        allowed_systems: serde_json::Value,
    ) -> AppResult<AdminEntity> {
        let row = sqlx::query_as::<_, AdminEntity>(
            r#"
            INSERT INTO admins (username, email, password_hash, role, allowed_systems)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, role, allowed_systems, status, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create admin: {}", e)))?;

        Ok(row)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<serde_json::Value>,
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
            "#
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .bind(status)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update admin: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Admin #{} not found", id)))?;

        Ok(row)
    }
}

use crate::entities::ModelRecordEntity;
use crate::orm::{Dialect, OrmInsert, OrmSelect, OrmUpdate, bind_pg_query, bind_pg_query_as};
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

        // 1. Total count query via ORM
        let (count_sql, count_binds) = OrmSelect::from_table("model_records")
            .count()
            .where_eq("system_id", system_slug)
            .where_eq("model_slug", model_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);

        let count_query = bind_pg_query_as(
            sqlx::query_as::<_, (i64,)>(sqlx::AssertSqlSafe(count_sql.as_str())),
            count_binds,
        );
        let total_row = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Count query failed: {}", e)))?;
        let total = total_row.0 as u64;

        // 2. Select paginated records via ORM
        let (select_sql, select_binds) = OrmSelect::from_table("model_records")
            .columns(&[
                "id",
                "system_id",
                "model_slug",
                "data",
                "created_at",
                "updated_at",
                "deleted_at",
            ])
            .where_eq("system_id", system_slug)
            .where_eq("model_slug", model_slug)
            .where_null("deleted_at")
            .order_by_desc("created_at")
            .limit(page_size as i64)
            .offset(offset as i64)
            .to_sql(Dialect::Postgres);

        let select_query = bind_pg_query_as(
            sqlx::query_as::<_, ModelRecordEntity>(sqlx::AssertSqlSafe(select_sql.as_str())),
            select_binds,
        );
        let records = select_query
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
        let (sql, binds) = OrmSelect::from_table("model_records")
            .columns(&[
                "id",
                "system_id",
                "model_slug",
                "data",
                "created_at",
                "updated_at",
                "deleted_at",
            ])
            .where_eq("id", id)
            .where_eq("system_id", system_slug)
            .where_eq("model_slug", model_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);

        let query = bind_pg_query_as(
            sqlx::query_as::<_, ModelRecordEntity>(sqlx::AssertSqlSafe(sql.as_str())),
            binds,
        );
        let record = query
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
        let (sql, binds) = OrmInsert::into_table("model_records")
            .value("system_id", system_slug)
            .value("model_slug", model_slug)
            .value("data", data)
            .returning(&[
                "id",
                "system_id",
                "model_slug",
                "data",
                "created_at",
                "updated_at",
                "deleted_at",
            ])
            .to_sql(Dialect::Postgres);

        let query = bind_pg_query_as(
            sqlx::query_as::<_, ModelRecordEntity>(sqlx::AssertSqlSafe(sql.as_str())),
            binds,
        );
        let record = query
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
        let (sql, binds) = OrmUpdate::table("model_records")
            .set("data", data)
            .set_now("updated_at")
            .where_eq("id", id)
            .where_eq("system_id", system_slug)
            .where_eq("model_slug", model_slug)
            .where_null("deleted_at")
            .returning(&[
                "id",
                "system_id",
                "model_slug",
                "data",
                "created_at",
                "updated_at",
                "deleted_at",
            ])
            .to_sql(Dialect::Postgres);

        let query = bind_pg_query_as(
            sqlx::query_as::<_, ModelRecordEntity>(sqlx::AssertSqlSafe(sql.as_str())),
            binds,
        );
        let record = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Update record failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        Ok(record)
    }

    async fn delete(&self, system_slug: &str, model_slug: &str, id: i64) -> AppResult<()> {
        let (sql, binds) = OrmUpdate::table("model_records")
            .set_now("deleted_at")
            .where_eq("id", id)
            .where_eq("system_id", system_slug)
            .where_eq("model_slug", model_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);

        let query = bind_pg_query(sqlx::query(sqlx::AssertSqlSafe(sql.as_str())), binds);
        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Delete record failed: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        Ok(())
    }
}

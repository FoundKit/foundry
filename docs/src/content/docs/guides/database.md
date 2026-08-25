---
title: Database & Custom Storage Guide
description: Complete guide to database operations in Foundry, including native SQLx queries, transactions, Zero-DDL dynamic records, and migrations.
---

# Database & Custom Storage Guide

In Foundry applications, developers have complete freedom to interact with the database using either **Custom SQL Queries (via SQLx)** or **Zero-DDL Dynamic Models (via RecordStore)**.

---

## 1. Overview: Two Storage Paradigms

| Storage Paradigm | Best For | Implementation |
|---|---|---|
| **Native SQL Tables (Custom Domain)** | Complex business relationships, high-performance structured joins, heavy analytical queries | Custom SQL tables created in `migrations/` + `sqlx` queries |
| **Zero-DDL Dynamic Models** | Visual entity management in Admin UI, rapid iteration without DDL locks, auto-generated REST CRUD | Built-in `model_records` table + `RecordStore` API |

---

## 2. Accessing the Database Pool (`DbPool`)

In your subsystem controllers, you can inject the database connection pool directly via Axum's `Extension(db)` extractor:

```rust
use axum::{extract::Extension, routing::get, Json, Router};
use foundry::prelude::*;

pub fn build_routes() -> Router {
    Router::new().route("/stats", get(handle_get_stats))
}

pub async fn handle_get_stats(
    Extension(ctx): Extension<SystemContext>,
    Extension(db): Extension<DbPool>,
) -> AppResult<Json<ApiResponse<StatsResponse>>> {
    let stats = StatsService::calculate(&ctx, &db).await?;
    Ok(Json(ApiResponse::success(stats)))
}
```

---

## 3. Executing Native SQL Queries with SQLx

Foundry's `DbPool` is a standard `sqlx::PgPool`. You can write type-safe queries using `sqlx`:

### Step 1: Define Your Data Struct

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomArticle {
    pub id: i64,
    pub system_slug: String,
    pub title: String,
    pub content: String,
    pub views: i32,
    pub created_at: DateTime<Utc>,
}
```

### Step 2: Querying Records

```rust
use foundry::prelude::*;
use crate::models::CustomArticle;

pub struct ArticleRepository;

impl ArticleRepository {
    /// Fetch all articles for the current subsystem
    pub async fn list_by_system(
        db: &DbPool,
        system_slug: &str,
    ) -> AppResult<Vec<CustomArticle>> {
        let rows = sqlx::query_as::<_, CustomArticle>(
            "SELECT id, system_slug, title, content, views, created_at 
             FROM articles 
             WHERE system_slug = $1 
             ORDER BY created_at DESC"
        )
        .bind(system_slug)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }

    /// Insert a new article
    pub async fn create(
        db: &DbPool,
        system_slug: &str,
        title: &str,
        content: &str,
    ) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO articles (system_slug, title, content, views, created_at)
             VALUES ($1, $2, $3, 0, NOW())
             RETURNING id"
        )
        .bind(system_slug)
        .bind(title)
        .bind(content)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.0)
    }
}
```

---

## 4. Handling Database Transactions (ACID)

For operations modifying multiple records atomically, use database transactions:

```rust
use foundry::prelude::*;

pub async fn transfer_balance(
    db: &DbPool,
    from_user: i64,
    to_user: i64,
    amount: i64,
) -> AppResult<()> {
    // 1. Begin transaction
    let mut tx = db.begin().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 2. Deduct from sender
    let deduct_res = sqlx::query(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2 AND balance >= $1"
    )
    .bind(amount)
    .bind(from_user)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if deduct_res.rows_affected() == 0 {
        return Err(AppError::BadRequest("Insufficient balance".to_string()));
    }

    // 3. Credit receiver
    sqlx::query(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(to_user)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 4. Commit transaction
    tx.commit().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}
```

---

## 5. Working with Zero-DDL Dynamic Models

Foundry provides a built-in JSONB storage engine (`RecordStore`) that allows saving and querying dynamic schemas without running SQL migrations:

```rust
use foundry::prelude::*;
use foundry_storage::models::RecordStore;
use serde_json::json;

pub async fn save_dynamic_post(
    db: &DbPool,
    ctx: &SystemContext,
    title: &str,
    content: &str,
) -> AppResult<i64> {
    let payload = json!({
        "title": title,
        "content": content,
        "status": "published",
        "tags": ["rust", "foundry"]
    });

    // Automatically stored in `model_records` table partitioned by system_slug & model_slug
    let record_id = RecordStore::create(
        db,
        &ctx.system_slug,
        "posts",
        payload,
    )
    .await?;

    Ok(record_id)
}
```

---

## 6. Accessing System Configurations (`ConfigStore`)

Read dynamic system configurations defined in the Admin UI:

```rust
use foundry::prelude::*;
use foundry_storage::configs::ConfigStore;

pub async fn get_system_settings(
    db: &DbPool,
    ctx: &SystemContext,
) -> AppResult<bool> {
    let configs = ConfigStore::get_aggregated(db, &ctx.system_slug).await?;
    
    // Check if maintenance mode is enabled
    let in_maintenance = configs
        .get("maintenance_mode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(in_maintenance)
}
```

---

## 7. Tenant-Isolated Redis Caching

When Redis is enabled, inject `Extension(redis): Extension<RedisPool>` and use `ctx.redis_key(...)` to isolate keys per subsystem:

```rust
use axum::extract::Extension;
use foundry::prelude::*;

pub async fn get_cached_item(
    ctx: &SystemContext,
    redis: &RedisPool,
    item_id: &str,
) -> AppResult<Option<String>> {
    // Generates key: "foundry:{system_slug}:items:{item_id}"
    let cache_key = ctx.redis_key(&format!("items:{}", item_id));

    let mut conn = redis.get().await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let val: Option<String> = redis::cmd("GET")
        .arg(&cache_key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(val)
}
```

---

## 8. Writing Custom Database Migrations

Place your custom domain `.sql` migration files in the `migrations/` directory of your project:

```text
my-app/
└── migrations/
    ├── 001_create_articles_table.sql
    └── 002_create_orders_table.sql
```

#### Example: `migrations/001_create_articles_table.sql`

```sql
CREATE TABLE IF NOT EXISTS articles (
    id BIGSERIAL PRIMARY KEY,
    system_slug VARCHAR(64) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    views INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_articles_system_slug ON articles(system_slug);
```

When you start your application with `AUTO_MIGRATE=true` or run `foundry migrate`, baseline schema tables and your custom migrations will be safely applied.

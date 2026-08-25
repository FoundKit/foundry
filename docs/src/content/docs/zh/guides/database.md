---
title: 数据库与自定义存储开发
description: Foundry 数据库与自定义存储开发完整指南，涵盖原生 SQLx 查询、事务控制、Zero-DDL 动态模型读写与业务迁移。
---

# 数据库与自定义存储开发指南

在 Foundry 应用开发中，开发者可以根据业务场景自由选择 **原生 SQLx 关系型存储** 或 **Zero-DDL 动态模型引擎 (RecordStore)**。

---

## 1. 存储选型与场景对比

| 存储模式 | 适用场景 | 开发实现方式 |
|---|---|---|
| **原生业务表 (SQLx)** | 复杂业务关联关系、高并发结构化 Join 查询、财务级事务 | 在 `migrations/` 编写 SQL DDL + 使用 `sqlx` 强类型查询 |
| **Zero-DDL 动态模型** | 后台可视化配置字段、快速原型迭代、无锁免停机加字段、自动 RESTful CRUD | 使用内置的 `model_records` 引擎与 `RecordStore` API |

---

## 2. 在自定义代码中获取数据库连接池 (`DbPool`)

在业务子系统的控制器函数中，可以通过 Axum 的 `Extension(db)` 提取器直接注入数据库连接池：

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

## 3. 使用 SQLx 执行原生数据库查询

Foundry 的 `DbPool` 为标准的 `sqlx::PgPool`。你可以使用类型安全的 SQLx 进行原生查询：

### 步骤 1: 定义数据结构体

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

### 步骤 2: 编写数据库 Repository 逻辑

```rust
use foundry::prelude::*;
use crate::models::CustomArticle;

pub struct ArticleRepository;

impl ArticleRepository {
    /// 按当前子系统查询文章列表
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

    /// 插入一条新文章
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

## 4. 数据库事务操作 (ACID)

当业务涉及多个数据表的原子变更时，使用数据库事务：

```rust
use foundry::prelude::*;

pub async fn transfer_balance(
    db: &DbPool,
    from_user: i64,
    to_user: i64,
    amount: i64,
) -> AppResult<()> {
    // 1. 开启事务
    let mut tx = db.begin().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 2. 扣减转出方账户
    let deduct_res = sqlx::query(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2 AND balance >= $1"
    )
    .bind(amount)
    .bind(from_user)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if deduct_res.rows_affected() == 0 {
        return Err(AppError::BadRequest("账户余额不足".to_string()));
    }

    // 3. 增加收款方账户
    sqlx::query(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(to_user)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 4. 提交事务 (若中途出错函数返回 Err 则自动 Rollback)
    tx.commit().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}
```

---

## 5. 操作 Zero-DDL 动态数据模型

Foundry 内置了免 DDL 动态模型存储（`RecordStore`），可在不执行 DDL 迁移的情况下安全读写任意动态 JSON Schema：

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

    // 自动保存至 model_records 表，按 system_slug 与 model_slug 严格隔离
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

## 6. 读取系统动态配置 (`ConfigStore`)

读取管理员在后台动态配置的系统参数：

```rust
use foundry::prelude::*;
use foundry_storage::configs::ConfigStore;

pub async fn get_system_settings(
    db: &DbPool,
    ctx: &SystemContext,
) -> AppResult<bool> {
    let configs = ConfigStore::get_aggregated(db, &ctx.system_slug).await?;
    
    // 读取维护模式开关
    let in_maintenance = configs
        .get("maintenance_mode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(in_maintenance)
}
```

---

## 7. 租户隔离的 Redis 缓存读写

当启用 Redis 时，提取 `Extension(redis): Extension<RedisPool>`，并使用 `ctx.redis_key(...)` 自动添加子系统隔离前缀：

```rust
use axum::extract::Extension;
use foundry::prelude::*;

pub async fn get_cached_item(
    ctx: &SystemContext,
    redis: &RedisPool,
    item_id: &str,
) -> AppResult<Option<String>> {
    // 自动生成带命名空间的 Key: "foundry:{system_slug}:items:{item_id}"
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

## 8. 编写应用专属数据库迁移脚本

将业务专属的 `.sql` 迁移文件放置在工程根目录的 `migrations/` 目录下：

```text
my-app/
└── migrations/
    ├── 001_create_articles_table.sql
    └── 002_create_orders_table.sql
```

#### 示例: `migrations/001_create_articles_table.sql`

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

启动应用时若配置 `AUTO_MIGRATE=true`，或手动执行 `foundry migrate`，框架将自动应用迁移脚本。

---
title: "02. Storage Layer & Model Engine (foundry_storage)"
description: "Stage 2: In-depth analysis of database pool, Redis session caching, and RecordStore dynamic models."
---

> **Codebase Navigation**：⬅️ Previous: [01. Core Types & Trait Abstractions (foundry_core)](./01-core/) · [📋 Overview & Guide](./) · ➡️ Next: [03. Authentication & RBAC (foundry_auth)](./03-auth/)

# Foundry Storage 代码深度分析

> 第二阶段：理解存储层实现
>
> Crate: `foundry_storage`
>
> 阅读时间：60-90 分钟

---

## 概述

`foundry_storage` 是 Foundry 框架的**数据持久化层**，实现了：
- **PostgreSQL 连接池管理**
- **Redis 连接管理**
- **Zero-DDL 动态模型系统**（核心特性）
- **系统、管理员、配置、审计日志的 CRUD**
- **租户隔离的数据访问**

**依赖关系：**
```
foundry_core (基础类型) → foundry_storage (数据访问)
```

**关键依赖：**
- `sqlx` - 异步 PostgreSQL 客户端（编译时 SQL 验证）
- `redis` - 异步 Redis 客户端
- `serde_json` - JSON 序列化/反序列化
- `chrono` - 日期时间处理
- `uuid` - UUID 生成

---

## 架构总览

```
foundry_storage
├── db.rs              # PostgreSQL 连接池初始化
├── redis_client.rs    # Redis 连接管理
├── entities.rs        # 数据库实体定义（FromRow）
├── systems.rs         # SystemStore：子系统 CRUD
├── models.rs          # ModelStore + RecordStore：动态模型核心
├── admins.rs          # AdminStore：管理员 CRUD
├── configs.rs         # ConfigStore：系统配置 KV 存储
├── audit.rs           # AuditStore：审计日志
└── lib.rs             # 统一导出
```

---

## 1. entities.rs - 数据库实体定义

### 1.1 设计模式：FromRow 派生

所有实体都使用 `#[derive(FromRow)]`，让 SQLx 自动将数据库行映射到 Rust 结构体。

```rust
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemEntity {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,  // 软删除标记
}
```

**FromRow 优势：**
- 编译时类型检查
- 自动字段映射（按名称）
- 支持 Option<T>（NULL 值）
- 支持自定义类型转换

### 1.2 核心实体解析

#### A. SystemEntity vs SystemItem

```rust
// 基础实体（直接映射表）
pub struct SystemEntity {
    pub id: Uuid,
    pub slug: String,
    // ... 基础字段
}

// 扩展实体（JOIN 聚合统计）
pub struct SystemItem {
    pub id: Uuid,
    pub slug: String,
    // ... 基础字段
    #[sqlx(default)]  // ← 如果 SQL 未返回此字段，使用默认值
    pub models_count: i64,
    #[sqlx(default)]
    pub configs_count: i64,
    #[sqlx(default)]
    pub records_count: i64,
}
```

**设计理念：** 
- `SystemEntity`：轻量级，基础 CRUD
- `SystemItem`：管理后台展示，带统计信息
- `SystemStats`：详细统计（包含审计日志数量）

#### B. 软删除模式

```rust
pub deleted_at: Option<DateTime<Utc>>
```

**原理：**
```sql
-- 删除操作不真正删除，而是标记时间戳
UPDATE systems SET deleted_at = NOW() WHERE id = $1;

-- 查询时过滤已删除记录
SELECT * FROM systems WHERE deleted_at IS NULL;
```

**优势：**
- 数据可恢复
- 保留审计痕迹
- 外键关联不会破坏

#### C. ModelRecordEntity - 动态数据的核心

```rust
pub struct ModelRecordEntity {
    pub id: i64,
    pub system_id: String,      // 租户隔离
    pub model_slug: String,      // 模型标识
    pub data: serde_json::Value, // ← JSON 存储任意结构
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

**核心设计：** `data` 字段使用 PostgreSQL 的 `JSONB` 类型，存储任意结构的 JSON 数据。

**数据库表结构：**
```sql
CREATE TABLE model_records (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(64) NOT NULL,
    model_slug VARCHAR(48) NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    INDEX idx_system_model (system_id, model_slug),
    INDEX idx_data_gin (data)  -- GIN 索引支持 JSON 查询
);
```

#### D. AdminEntity - 管理员实体

```rust
pub struct AdminEntity {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,            // Argon2id 哈希
    pub role: String,                      // super_admin | admin | topic_admin
    pub allowed_systems: serde_json::Value, // RBAC：允许访问的系统列表
    pub status: i16,                       // 0=禁用, 1=启用
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**RBAC 设计：**
```json
// super_admin：allowed_systems = []（空数组表示全部权限）
// admin：allowed_systems = []
// topic_admin：allowed_systems = ["blog", "shop"]（只能访问指定系统）
```

#### E. AuditLogEntity - 审计日志

```rust
pub struct AuditLogEntity {
    pub id: i64,
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub system_slug: Option<String>,
    pub method: String,              // GET, POST, PUT, DELETE
    pub path: String,                 // /api/v1/s/blog/articles
    pub action_name: Option<String>,  // "create_article"
    pub headers: serde_json::Value,
    pub query_params: Option<String>,
    pub body_params: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status_code: Option<i16>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

**记录内容：**
- 谁（admin_id）
- 什么时候（created_at）
- 做了什么（method + path + action_name）
- 结果如何（status_code）
- 耗时多久（duration_ms）

### 1.3 统计聚合实体

#### PlatformSummary - 平台级统计

```rust
pub struct PlatformSummary {
    pub total_systems: u64,
    pub active_systems: u64,
    pub total_models: u64,
    pub total_records: u64,
    pub total_admins: u64,
    pub total_audit_logs: u64,
}
```

**用途：** Dashboard 首页展示平台整体概况

---

## 2. db.rs - PostgreSQL 连接池

### 2.1 连接池初始化

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

pub type DbPool = PgPool;  // 类型别名

pub async fn init_db_pool(
    database_url: &str,
    max_connections: u32
) -> AppResult<DbPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))  // 获取连接超时
        .connect(database_url)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect: {}", e)))
}
```

**关键参数：**
- `max_connections`：连接池大小（默认推荐：CPU 核心数 * 2）
- `acquire_timeout`：从池中获取连接的超时时间

**连接池工作原理：**
```
请求 1 → 从池中获取连接 → 执行查询 → 归还连接
请求 2 → 从池中获取连接 → 执行查询 → 归还连接
...
如果池满 → 等待 5 秒 → 超时返回错误
```

### 2.2 数据库迁移

```rust
pub async fn run_migrations(pool: &DbPool) -> AppResult<()> {
    info!("Running database initialization migrations...");
    let init_sql = include_str!("../migrations/init.sql");
    sqlx::raw_sql(init_sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;
    info!("Database migration completed successfully.");
    Ok(())
}
```

**`include_str!` 宏：** 在编译时将 SQL 文件内容嵌入二进制文件。

**迁移脚本示例（migrations/init.sql）：**
```sql
-- 创建表（如果不存在）
CREATE TABLE IF NOT EXISTS systems (...);
CREATE TABLE IF NOT EXISTS models (...);
CREATE TABLE IF NOT EXISTS model_records (...);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_records_system_model 
ON model_records(system_id, model_slug);

-- 创建 GIN 索引用于 JSON 查询
CREATE INDEX IF NOT EXISTS idx_records_data_gin 
ON model_records USING GIN(data);

-- 插入初始超级管理员
INSERT INTO admins (username, password_hash, role)
VALUES ('admin', '$argon2id$...', 'super_admin')
ON CONFLICT (username) DO NOTHING;
```

---

## 3. redis_client.rs - Redis 连接管理

### 3.1 ConnectionManager 模式

```rust
use redis::aio::ConnectionManager;

pub type RedisPool = ConnectionManager;

pub async fn init_redis(redis_url: &str) -> AppResult<RedisPool> {
    let client = redis::Client::open(redis_url)
        .map_err(|e| AppError::Internal(format!("Invalid Redis URL: {}", e)))?;

    let manager = ConnectionManager::new(client)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect: {}", e)))?;

    info!("Redis connection established successfully.");
    Ok(manager)
}
```

**ConnectionManager vs Connection：**

| 特性 | Connection | ConnectionManager |
|------|-----------|-------------------|
| 连接方式 | 单个连接 | 自动重连的连接管理器 |
| 网络断开 | 抛出错误 | 自动重连 |
| 并发性 | 需要手动管理 | 内部处理 |
| 推荐场景 | 简单脚本 | 生产环境 |

### 3.2 使用示例

```rust
use redis::AsyncCommands;

// 设置缓存
let _: () = redis_pool.set("key", "value").await?;
redis_pool.expire("key", 3600).await?;  // 1小时过期

// 获取缓存
let value: Option<String> = redis_pool.get("key").await?;

// 删除缓存
let _: () = redis_pool.del("key").await?;

// Hash 操作
redis_pool.hset("user:123", "name", "Alice").await?;
let name: String = redis_pool.hget("user:123", "name").await?;
```

---

## 4. systems.rs - 子系统存储

### 4.1 SystemStore 设计模式

```rust
pub struct SystemStore;  // Zero-sized type (无状态)

impl SystemStore {
    // 所有方法都是静态方法（&self 不需要）
    pub async fn list(pool: &DbPool) -> AppResult<Vec<SystemEntity>> {
        // ...
    }
}
```

**为什么使用 Unit Struct？**
- 无需实例化（`SystemStore::list(&pool)`）
- 作为命名空间使用
- 零运行时开销

### 4.2 分页查询实现

```rust
pub async fn list_paginated(
    pool: &DbPool,
    query: SystemQuery,
    allowed_systems: Option<&[String]>,
) -> AppResult<PaginatedData<SystemItem>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * page_size;

    // 1. 统计总数
    let count_row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM systems WHERE deleted_at IS NULL AND ..."
    )
    .bind(...)
    .fetch_one(pool)
    .await?;

    let total = count_row.0 as u64;

    // 2. 获取分页数据 + 聚合统计
    let rows = sqlx::query_as::<_, SystemItem>(
        r#"
        SELECT
            s.*,
            (SELECT COUNT(*) FROM models m WHERE m.system_id = s.slug) AS models_count,
            (SELECT COUNT(*) FROM system_configs c WHERE c.system_id = s.slug) AS configs_count,
            (SELECT COUNT(*) FROM model_records r WHERE r.system_id = s.slug) AS records_count
        FROM systems s
        WHERE s.deleted_at IS NULL
        ORDER BY s.created_at ASC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    Ok(PaginatedData::new(rows, page, page_size, total))
}
```

**性能考量：**
- 子查询会有 N+1 问题吗？→ 每个系统一次查询，但数量有限
- 优化方案：使用 LEFT JOIN + GROUP BY（复杂度更高，可读性差）

### 4.3 动态 WHERE 条件

```rust
let slug_filter = query.slug.as_ref().map(|s| format!("%{}%", s.trim().to_lowercase()));

// SQL 中：
WHERE ($1::varchar IS NULL OR LOWER(slug) LIKE $1)
```

**原理：**
- 如果 `slug_filter = None`，绑定 `NULL`，条件变为 `NULL IS NULL`（永远为真，跳过过滤）
- 如果 `slug_filter = Some("%blog%")`，条件变为 `LOWER(slug) LIKE '%blog%'`

**优势：** 一个 SQL 语句处理多种查询组合，避免大量 if-else。

### 4.4 RBAC 过滤（allowed_systems）

```rust
pub async fn list_paginated(
    pool: &DbPool,
    query: SystemQuery,
    allowed_systems: Option<&[String]>,  // ← RBAC 传入允许的系统列表
) -> AppResult<PaginatedData<SystemItem>> {
    let has_allowed_check = allowed_systems.is_some();
    let allowed_list = allowed_systems.unwrap_or(&[]);

    let rows = sqlx::query_as::<_, SystemItem>(
        r#"
        WHERE deleted_at IS NULL
          AND ($1::bool = false OR slug = ANY($2))  -- ← 动态 RBAC 过滤
        "#
    )
    .bind(has_allowed_check)
    .bind(allowed_list)
    .fetch_all(pool)
    .await?;
}
```

**场景：**
- **super_admin**：`allowed_systems = None` → 查询所有系统
- **topic_admin**：`allowed_systems = Some(&["blog", "shop"])` → 只查询这两个系统

---

## 5. models.rs - 动态模型核心（⭐ 重点）

### 5.1 模型定义层（ModelStore）

#### A. 模型元数据管理

```rust
pub struct ModelStore;

impl ModelStore {
    // 列出子系统下的所有模型
    pub async fn list_models(
        pool: &DbPool,
        system_slug: &str
    ) -> AppResult<Vec<ModelEntity>> {
        sqlx::query_as(
            "SELECT * FROM models WHERE system_id = $1 AND deleted_at IS NULL"
        )
        .bind(system_slug)
        .fetch_all(pool)
        .await?
    }

    // 创建新模型
    pub async fn create_model(
        pool: &DbPool,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        let default_perms = json!({
            "public_read": false,
            "public_write": false,
            "auth_read": true,
            "auth_write": false
        });

        sqlx::query_as(
            "INSERT INTO models (...) VALUES (...) RETURNING *"
        )
        .bind(system_slug)
        .bind(slug)
        .bind(name)
        .bind(permissions.unwrap_or(default_perms))
        .fetch_one(pool)
        .await?
    }
}
```

#### B. 模型字段定义

```rust
pub struct ModelFieldEntity {
    pub id: i64,
    pub model_id: i64,
    pub name: String,           // 字段名（如 "title"）
    pub label: String,           // 显示标签（如 "文章标题"）
    pub field_type: String,      // "string", "integer", "boolean"...
    pub is_required: bool,
    pub default_value: Option<Value>,
    pub options: Value,          // 字段选项（如枚举值、验证规则）
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ModelStore {
    pub async fn add_field(
        pool: &DbPool,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        // INSERT INTO model_fields ...
    }
}
```

**示例：定义一个 "articles" 模型**

```rust
// 1. 创建模型
let model = ModelStore::create_model(
    &pool,
    "blog",
    "articles",
    "文章",
    Some("博客文章内容"),
    None
).await?;

// 2. 添加字段
ModelStore::add_field(&pool, model.id, "title", "标题", "string", true, None, None, 1).await?;
ModelStore::add_field(&pool, model.id, "content", "内容", "richtext", true, None, None, 2).await?;
ModelStore::add_field(&pool, model.id, "views", "浏览量", "integer", false, Some(json!(0)), None, 3).await?;
ModelStore::add_field(&pool, model.id, "published", "已发布", "boolean", false, Some(json!(false)), None, 4).await?;
```

### 5.2 记录数据层（RecordStore）- 核心实现

#### A. 记录验证

```rust
impl RecordStore {
    pub fn validate_record(
        fields: &[ModelFieldEntity],
        data: &Value
    ) -> AppResult<()> {
        let obj = data.as_object()
            .ok_or_else(|| AppError::Validation("Must be JSON object".into()))?;

        for field in fields {
            let val = obj.get(&field.name);

            // 1. 必填字段检查
            if field.is_required && (val.is_none() || val == Some(&Value::Null)) {
                return Err(AppError::Validation(
                    format!("Field '{}' is required", field.name)
                ));
            }

            // 2. 类型检查
            if let Some(val) = val && !val.is_null() {
                let ft = field.field_type.parse::<FieldType>()?;
                match ft {
                    FieldType::String | FieldType::Richtext => {
                        if !val.is_string() {
                            return Err(AppError::Validation(
                                format!("Field '{}' must be string", field.name)
                            ));
                        }
                    }
                    FieldType::Integer => {
                        if !val.is_i64() && !val.is_u64() {
                            return Err(AppError::Validation(
                                format!("Field '{}' must be integer", field.name)
                            ));
                        }
                    }
                    // ... 其他类型检查
                }
            }
        }
        Ok(())
    }
}
```

**测试用例：**

```rust
#[test]
fn test_record_validation_success() {
    let fields = vec![
        sample_field("title", "Title", "string", true),
        sample_field("price", "Price", "number", true),
    ];

    let valid = json!({
        "title": "Product A",
        "price": 99.99
    });

    assert!(RecordStore::validate_record(&fields, &valid).is_ok());
}

#[test]
fn test_record_validation_missing_required() {
    let fields = vec![sample_field("title", "Title", "string", true)];
    let missing = json!({ "price": 100 });
    assert!(RecordStore::validate_record(&fields, &missing).is_err());
}
```

#### B. CRUD 操作

##### Create（创建记录）

```rust
pub async fn create(
    pool: &DbPool,
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
    .fetch_one(pool)
    .await?;

    Ok(record)
}

// 使用示例：
let article_data = json!({
    "title": "Hello World",
    "content": "<p>First post</p>",
    "views": 0,
    "published": true
});

let record = RecordStore::create(&pool, "blog", "articles", article_data).await?;
println!("Created record #{}", record.id);
```

##### Read（查询记录）

```rust
// 分页列表
pub async fn list(
    pool: &DbPool,
    system_slug: &str,
    model_slug: &str,
    query: RecordQuery,
) -> AppResult<PaginatedData<ModelRecordEntity>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    // 统计总数
    let total_row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM model_records 
         WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL"
    )
    .bind(system_slug)
    .bind(model_slug)
    .fetch_one(pool)
    .await?;

    // 分页查询
    let records = sqlx::query_as::<_, ModelRecordEntity>(
        "SELECT * FROM model_records 
         WHERE system_id = $1 AND model_slug = $2 AND deleted_at IS NULL
         ORDER BY created_at DESC
         LIMIT $3 OFFSET $4"
    )
    .bind(system_slug)
    .bind(model_slug)
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    Ok(PaginatedData::new(records, page, page_size, total_row.0 as u64))
}

// 单条查询
pub async fn get_by_id(
    pool: &DbPool,
    system_slug: &str,
    model_slug: &str,
    id: i64,
) -> AppResult<ModelRecordEntity> {
    sqlx::query_as::<_, ModelRecordEntity>(
        "SELECT * FROM model_records 
         WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL"
    )
    .bind(id)
    .bind(system_slug)
    .bind(model_slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))
}
```

##### Update（更新记录）

```rust
pub async fn update(
    pool: &DbPool,
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
        RETURNING *
        "#,
    )
    .bind(data)
    .bind(id)
    .bind(system_slug)
    .bind(model_slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

    Ok(record)
}

// 使用示例：
let updated_data = json!({
    "title": "Hello World (Updated)",
    "views": 100  // 更新浏览量
});

let record = RecordStore::update(&pool, "blog", "articles", 1, updated_data).await?;
```

##### Delete（软删除）

```rust
pub async fn delete(
    pool: &DbPool,
    system_slug: &str,
    model_slug: &str,
    id: i64,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE model_records SET deleted_at = NOW() 
         WHERE id = $1 AND system_id = $2 AND model_slug = $3 AND deleted_at IS NULL"
    )
    .bind(id)
    .bind(system_slug)
    .bind(model_slug)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Record #{} not found", id)));
    }

    Ok(())
}
```

### 5.3 租户隔离的关键点

**每个 CRUD 操作都强制传入 `system_slug` 和 `model_slug`：**

```rust
RecordStore::create(&pool, "blog", "articles", data)  // ← 必须显式指定
RecordStore::get_by_id(&pool, "blog", "articles", 1)
```

**SQL 查询自动添加过滤条件：**
```sql
WHERE system_id = 'blog' AND model_slug = 'articles'
```

**这保证了：**
- `blog` 系统无法访问 `shop` 系统的数据
- 不同模型的数据严格隔离

---

## 6. admins.rs - 管理员存储

### 6.1 基础 CRUD

```rust
pub struct AdminStore;

impl AdminStore {
    // 列出所有管理员
    pub async fn list(pool: &DbPool) -> AppResult<Vec<AdminEntity>> {
        sqlx::query_as("SELECT * FROM admins ORDER BY created_at ASC")
            .fetch_all(pool)
            .await?
    }

    // 根据用户名查询（登录时使用）
    pub async fn get_by_username(pool: &DbPool, username: &str) -> AppResult<AdminEntity> {
        sqlx::query_as("SELECT * FROM admins WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Admin '{}' not found", username)))
    }

    // 创建管理员
    pub async fn create(
        pool: &DbPool,
        username: &str,
        email: Option<&str>,
        password_hash: &str,  // ← 已经过 Argon2id 哈希
        role: &str,
        allowed_systems: Value,
    ) -> AppResult<AdminEntity> {
        sqlx::query_as(
            "INSERT INTO admins (...) VALUES (...) RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .fetch_one(pool)
        .await?
    }

    // 更新管理员（支持部分字段更新）
    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<Value>,
        status: Option<i16>,
    ) -> AppResult<AdminEntity> {
        sqlx::query_as(
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
            RETURNING *
            "#
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(allowed_systems)
        .bind(status)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Admin #{} not found", id)))
    }
}
```

### 6.2 COALESCE 技巧

```sql
password_hash = COALESCE($1, password_hash)
```

**含义：** 如果参数为 `NULL`，使用原有值；否则使用新值。

**效果：** 实现部分字段更新，无需重复传递所有字段。

```rust
// 只更新密码
AdminStore::update(
    &pool,
    admin_id,
    None,                    // email 不变
    Some(new_password_hash), // 更新密码
    None,                    // role 不变
    None,                    // allowed_systems 不变
    None,                    // status 不变
).await?;
```

---

## 7. configs.rs - 系统配置存储

### 7.1 配置 KV 设计

```rust
pub struct SystemConfigEntity {
    pub id: i64,
    pub system_id: String,      // 子系统 slug
    pub key: String,             // 配置键（如 "site_name"）
    pub label: String,           // 显示标签（如 "网站名称"）
    pub value_type: String,      // "string" | "integer" | "boolean" | "json"
    pub value: Option<Value>,    // 配置值
    pub options: Value,          // 选项（如枚举值、验证规则）
    pub sort_order: i32,         // 排序
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 7.2 聚合配置获取

```rust
impl ConfigStore {
    // 获取聚合的 JSON 对象
    pub async fn get_aggregated(
        pool: &DbPool,
        system_slug: &str
    ) -> AppResult<Value> {
        let configs = Self::list(pool, system_slug).await?;
        let mut map = serde_json::Map::new();

        for cfg in configs {
            let val = cfg.value.unwrap_or(Value::Null);
            map.insert(cfg.key, val);
        }

        Ok(Value::Object(map))
    }

    // 强类型配置（泛型）
    pub async fn get_typed<T: DeserializeOwned>(
        pool: &DbPool,
        system_slug: &str
    ) -> AppResult<T> {
        let agg = Self::get_aggregated(pool, system_slug).await?;
        serde_json::from_value::<T>(agg)
            .map_err(|e| AppError::Validation(format!("Failed to deserialize: {}", e)))
    }
}
```

**使用示例：**

```rust
// 定义配置结构体
#[derive(Deserialize)]
struct BlogConfig {
    site_name: String,
    posts_per_page: u32,
    enable_comments: bool,
}

// 获取强类型配置
let config: BlogConfig = ConfigStore::get_typed(&pool, "blog").await?;
println!("Site: {}, Per page: {}", config.site_name, config.posts_per_page);
```

### 7.3 批量更新配置

```rust
pub async fn update_values(
    pool: &DbPool,
    system_slug: &str,
    values: &serde_json::Map<String, Value>,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;  // ← 开启事务

    for (k, v) in values {
        sqlx::query(
            "UPDATE system_configs SET value = $1, updated_at = NOW() 
             WHERE system_id = $2 AND key = $3"
        )
        .bind(v)
        .bind(system_slug)
        .bind(k)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;  // ← 提交事务
    Ok(())
}

// 使用示例：
let updates = json!({
    "site_name": "New Blog",
    "posts_per_page": 15
});

ConfigStore::update_values(&pool, "blog", updates.as_object().unwrap()).await?;
```

**事务保证：** 所有配置更新要么全部成功，要么全部回滚。

---

## 8. audit.rs - 审计日志

### 8.1 日志插入

```rust
pub struct AuditLogInsert {
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub system_slug: Option<String>,
    pub method: String,
    pub path: String,
    pub action_name: Option<String>,
    pub headers: Value,
    pub query_params: Option<String>,
    pub body_params: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status_code: Option<i16>,
    pub duration_ms: Option<i32>,
}

impl AuditStore {
    pub async fn insert(pool: &DbPool, log: AuditLogInsert) -> AppResult<()> {
        sqlx::query("INSERT INTO audit_logs (...) VALUES (...)")
            .bind(log.admin_id)
            .bind(log.admin_username)
            // ... 绑定所有字段
            .execute(pool)
            .await?;
        Ok(())
    }
}
```

**在中间件中使用：**

```rust
// 请求前记录开始时间
let start_time = Instant::now();

// 执行请求处理
let response = next.run(request).await;

// 请求后记录日志
let duration_ms = start_time.elapsed().as_millis() as i32;
AuditStore::insert(&pool, AuditLogInsert {
    admin_id: Some(admin.id),
    method: "POST".to_string(),
    path: "/api/v1/s/blog/articles".to_string(),
    status_code: Some(201),
    duration_ms: Some(duration_ms),
    // ...
}).await?;
```

### 8.2 分页查询

```rust
pub async fn list(
    pool: &DbPool,
    query: AuditLogQuery,
) -> AppResult<PaginatedData<AuditLogEntity>> {
    // 支持按 admin_id、system_slug、method 过滤
    let logs = sqlx::query_as::<_, AuditLogEntity>(
        r#"
        SELECT * FROM audit_logs
        WHERE ($1::uuid IS NULL OR admin_id = $1)
          AND ($2::varchar IS NULL OR system_slug = $2)
          AND ($3::varchar IS NULL OR method = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#
    )
    .bind(query.admin_id)
    .bind(query.system_slug.as_deref())
    .bind(query.method.as_deref())
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    // ...
}
```

---

## 9. 关键设计模式总结

### 9.1 Repository Pattern（仓储模式）

```rust
pub struct SystemStore;  // Repository
pub struct ModelStore;
pub struct RecordStore;
```

**优势：**
- 数据访问逻辑集中
- 业务逻辑与数据库解耦
- 易于测试（可 mock）

### 9.2 Soft Delete（软删除）

```sql
deleted_at TIMESTAMPTZ
```

**实现：**
- 删除 → `UPDATE ... SET deleted_at = NOW()`
- 查询 → `WHERE deleted_at IS NULL`

**优势：** 数据可恢复、审计完整

### 9.3 Multi-Tenancy（多租户隔离）

```sql
WHERE system_id = $1 AND model_slug = $2
```

**强制租户参数：** 所有 CRUD 必须传 `system_slug`

### 9.4 Type-safe Database Access（类型安全）

```rust
sqlx::query_as::<_, SystemEntity>("SELECT ...")
```

**编译时检查：**
- SQL 语法正确性
- 列名与结构体字段匹配
- 类型兼容性

### 9.5 JSONB Schema-less（无模式 JSON）

```sql
data JSONB NOT NULL
```

**优势：**
- 无需预定义列
- 灵活的数据结构
- GIN 索引支持高效查询

---

## 10. 性能优化考虑

### 10.1 索引策略

```sql
-- 租户隔离索引
CREATE INDEX idx_records_system_model ON model_records(system_id, model_slug);

-- JSON 搜索索引
CREATE INDEX idx_records_data_gin ON model_records USING GIN(data);

-- 软删除过滤索引
CREATE INDEX idx_records_deleted ON model_records(deleted_at) WHERE deleted_at IS NULL;
```

### 10.2 连接池大小

```rust
max_connections: CPU 核心数 * 2
```

**原则：** 过大浪费资源，过小导致排队。

### 10.3 分页限制

```rust
page_size.clamp(1, 100)  // 最大 100 条/页
```

**防止：** 恶意请求大量数据导致内存溢出。

### 10.4 事务使用

```rust
let mut tx = pool.begin().await?;
// ... 多条 SQL
tx.commit().await?;
```

**适用场景：** 多表关联更新、需要原子性操作。

---

## 11. 测试覆盖

### 11.1 单元测试（models.rs）

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_record_validation_success() { /* ... */ }

    #[test]
    fn test_record_validation_missing_required() { /* ... */ }

    #[test]
    fn test_record_validation_type_mismatch() { /* ... */ }
}
```

### 11.2 集成测试建议

```rust
// tests/storage_integration.rs

#[tokio::test]
async fn test_create_and_fetch_record() {
    let pool = setup_test_db().await;
    
    let record = RecordStore::create(
        &pool,
        "test_system",
        "test_model",
        json!({"name": "Test"})
    ).await.unwrap();
    
    let fetched = RecordStore::get_by_id(
        &pool,
        "test_system",
        "test_model",
        record.id
    ).await.unwrap();
    
    assert_eq!(fetched.id, record.id);
}
```

---

## 12. 与其他 Crates 的交互

```
foundry_storage
    ↓ 被使用
┌───────────────────────────────────────┐
│ foundry_engine (handlers)             │
│   - 调用 SystemStore、RecordStore     │
│   - 实现 HTTP CRUD 端点               │
├───────────────────────────────────────┤
│ foundry_auth                          │
│   - 调用 AdminStore 验证登录          │
├───────────────────────────────────────┤
│ foundry (app.rs)                      │
│   - 初始化 DbPool 和 RedisPool        │
└───────────────────────────────────────┘
```

---

## 13. 下一步学习建议

理解 `foundry_storage` 后，推荐阅读顺序：

1. **foundry_auth** - 看 AdminStore 如何用于认证
2. **foundry_engine/handlers** - 看 RecordStore 如何实现 Auto-CRUD
3. **examples/blog_platform** - 看实际项目中的数据访问模式

---

## 14. 快速参考

### 常用操作

```rust
// 初始化
let pool = init_db_pool(&db_url, 10).await?;
let redis = init_redis(&redis_url).await?;

// 子系统查询
let systems = SystemStore::list(&pool).await?;
let system = SystemStore::get_by_slug(&pool, "blog").await?;

// 动态记录 CRUD
let record = RecordStore::create(&pool, "blog", "articles", data).await?;
let record = RecordStore::get_by_id(&pool, "blog", "articles", 1).await?;
let record = RecordStore::update(&pool, "blog", "articles", 1, data).await?;
RecordStore::delete(&pool, "blog", "articles", 1).await?;

// 配置管理
let config: BlogConfig = ConfigStore::get_typed(&pool, "blog").await?;
```

---

**✅ 第二阶段完成！** 你现在应该理解了：
- PostgreSQL 和 Redis 的连接管理
- 所有数据库实体的结构和用途
- Zero-DDL 动态模型的完整实现
- 租户隔离的实现机制
- Repository 模式的最佳实践

**准备好进入第三阶段了吗？** 下一步我们将深入 `foundry_auth`，看认证与授权如何实现。

---

### 📚 Stage Navigation
- ⬅️ **Previous Stage**: [01. Core Types & Trait Abstractions (foundry_core)](./01-core/)
- 📋 **Guide Overview**: [Return to Overview](./)
- ➡️ **Next Stage**: [03. Authentication & RBAC (foundry_auth)](./03-auth/)

---
title: 数据库与存储引擎开发指南
description: Foundry 多数据库 SPI 架构、PostgreSQL 与 MySQL 配置、Zero-DDL 动态模型引擎与容器化快速起步。
---

# 数据库与存储引擎开发指南

Foundry 采用全新的 **多数据库 SPI (Service Provider Interface) 插件化存储架构**。业务层、领域逻辑服务、AutoCRUD 引擎与 Axum 状态统一依赖 `Database` 门面与标准 Store Traits，**彻底实现底层网络驱动零泄露**。

系统原生支持 **PostgreSQL (14+)** 与 **MySQL (8.0+) / MariaDB (10.5+)**，并能根据连接串协议自动自适应路由驱动。

---

## 1. 核心架构与设计原则

```mermaid
flowchart TD
    subgraph AppLayer ["业务层 / AutoCRUD 引擎 / 控制器"]
        Handler["Axum Handler (Extension<AppState>)"]
        Call["纯方法调用: db.records() / db.models() / db.configs() / db.systems()"]
        Handler --> Call
    end

    subgraph FacadeLayer ["统一门面与 SPI 抽象 (foundry_storage)"]
        Call --> Facade["Database 门面"]
        Facade --> Registry["StorageRegistry (驱动路由)"]
    end

    subgraph Engines ["底层驱动实现 (按需编译)"]
        Registry -->|"postgres:// / postgresql://"| PgEngine["PostgresProvider (JSONB + GIN 倒排索引)"]
        Registry -->|"mysql:// / mariadb://"| MySqlEngine["MySqlProvider (通用单表 + 覆盖索引 + 延迟关联)"]
    end
```

### 核心特性
1. **面向方法调用，驱动零泄露**：上层业务代码无需关心底层是 PostgreSQL 还是 MySQL，统一通过 `state.db.records().create(...)`、`state.db.records().list(...)` 等 Trait 方法访问数据。
2. **智能协议自适应**：系统启动时自动分析 `DATABASE_URL` 的协议头并激活对应驱动：
   - `postgres://` 或 `postgresql://` $\rightarrow$ 自动激活 PostgreSQL
   - `mysql://` 或 `mariadb://` $\rightarrow$ 自动激活 MySQL / MariaDB
   - 亦可通过环境变量 `DATABASE_TYPE=mysql` 或 `DATABASE_TYPE=postgres` 显式强制指定。
3. **Cargo Feature 按需编译**：底层驱动通过 Feature Flags 控制，避免不必要的依赖编译膨胀。

---

## 2. 环境配置与 Feature Flags

### Cargo.toml 依赖配置

Foundry 默认开启 `postgres` 特性。若需使用 MySQL，只需在应用的 `Cargo.toml` 中开启 `mysql` 特性：

```toml
[dependencies]
# 仅使用 PostgreSQL (默认)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# 或者开启 MySQL 支持
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mysql"] }

# 或者同时支持 PostgreSQL 与 MySQL 双驱动
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["postgres", "mysql"] }
```

### 环境变量配置 (`.env`)

#### 使用 PostgreSQL：
```bash
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### 使用 MySQL / MariaDB：
```bash
DATABASE_URL=mysql://root:root@localhost:3306/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

> **提示**：当 `AUTO_MIGRATE=true` 时，服务启动时会自动检测数据库是否存在；若不存在会自动建库，并执行对应数据库方言的初始化迁移脚本（包括表结构、索引与默认超级管理员账号）。

---

## 3. 容器化一键启动 (Docker / Nerdctl / Podman)

项目根目录和 `dev/` 目录下提供了标准的 `compose.yml`，兼容 **Docker Compose**、**Nerdctl (containerd)** 与 **Podman Compose**：

### 启动数据库容器：

```bash
# 使用 Docker Compose
docker compose up -d postgres mysql redis

# 或使用 Nerdctl (containerd 环境)
nerdctl compose up -d postgres mysql redis

# 或使用 Podman Compose
podman-compose up -d postgres mysql redis
```

容器就绪后：
- **MySQL 8.4**：端口 `3306`，账号 `root`，密码 `root`，数据库 `foundry`。
- **PostgreSQL 18.6**：端口 `5432`，账号 `postgres`，密码 `postgrespassword`，数据库 `foundry`。
- **Redis 8.0**：端口 `6379`。

---

## 4. 业务代码中操作存储 (`Database` 门面)

在自定义控制器与业务逻辑中，通过 `Extension<AppState>` 获取统一的 `Database` 门面：

```rust
use axum::{extract::Extension, Json};
use foundry::prelude::*;
use serde_json::json;

pub async fn create_article_handler(
    Extension(state): Extension<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. 使用动态模型引擎写入数据 (Zero-DDL 存储)
    let record = state
        .db
        .records()
        .create(
            &ctx.system_slug,
            "articles",
            json!({
                "title": payload.get("title").and_then(|v| v.as_str()).unwrap_or("未命名"),
                "content": payload.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                "views": 0,
                "is_published": true
            }),
        )
        .await?;

    Ok(Json(ApiResponse::success(json!({
        "id": record.id,
        "data": record.data
    }))))
}
```

### Store Engines 常用方法速查：

| 领域模块 | 方法调用入口 | 常用方法 |
|---|---|---|
| **动态业务记录** | `db.records()` | `create()`, `get_by_id()`, `update()`, `delete()`, `list()` |
| **模型与字段元数据** | `db.models()` | `list_models()`, `get_model()`, `create_model()`, `list_fields()`, `add_field()` |
| **子系统租户** | `db.systems()` | `list()`, `list_paginated()`, `get_by_id()`, `get_by_slug()`, `create()`, `update()` |
| **动态系统配置** | `db.configs()` | `get()`, `set()`, `get_aggregated()`, `get_schema()` |
| **管理员与权限** | `db.admins()` | `get_by_id()`, `get_by_username()`, `create()`, `update()` |
| **审计追踪日志** | `db.audit()` | `insert()`, `list()` |

---

## 5. MySQL 通用单表架构与深度调优

针对 MySQL 无 GIN 倒排索引的特性，Foundry 落地了**单表通用存储 (Universal Single-Table) 架构**：

### 5.1 覆盖复合索引设计
```sql
CREATE TABLE IF NOT EXISTS model_records (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    model_id BIGINT NOT NULL,
    system_id VARCHAR(32) NOT NULL,
    model_slug VARCHAR(48) NOT NULL,
    data JSON NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_model_query (model_id, deleted_at, created_at DESC, id DESC),
    INDEX idx_system_query (system_id, deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### 5.2 延迟关联分页 (Deferred Join) 优化
当数据量达到百万级时，传统 `SELECT * LIMIT 100000, 20` 会因扫描大 JSON 列导致严重的内存与 IO 开销。Foundry 的 `MySqlRecordEngine` 自动采用**延迟关联分页**：

```sql
-- 1. 先仅扫描覆盖索引获取目标主键 ID (极速 Index Scan)
-- 2. 再通过主键回表拉取对应的完整 data JSON
SELECT r.id, r.system_id, r.model_slug, r.data, r.created_at, r.updated_at, r.deleted_at
FROM model_records r
INNER JOIN (
    SELECT id
    FROM model_records
    WHERE model_id = ? AND deleted_at IS NULL
    ORDER BY created_at DESC, id DESC
    LIMIT ? OFFSET ?
) t ON r.id = t.id
ORDER BY r.created_at DESC, r.id DESC;
```

此优化将深度分页的执行时间从秒级降至毫秒级，彻底消除性能瓶颈。

### 5.3 解决 MySQL 无 `RETURNING` 语法的原子重查
在数据插入时，PostgreSQL 支持 `INSERT ... RETURNING *`，而 MySQL 仅返回受影响行数与自增 ID。Foundry 在事务内部原子组合 `INSERT` $\rightarrow$ `last_insert_id()` $\rightarrow$ `SELECT`，对外统一呈现原子创建并返回完整实体的强一致体验。

---

## 6. 读取与注入强类型配置

通过 `db.get_typed_config::<T>(&system_slug, key)` 可以直接将动态配置反序列化为类型安全的结构体：

```rust
#[derive(Debug, Deserialize)]
pub struct PaymentConfig {
    pub api_key: String,
    pub merchant_id: String,
    pub sandbox: bool,
}

// 在控制器中读取
let pay_cfg: Option<PaymentConfig> = state
    .db
    .get_typed_config(&ctx.system_slug, "payment_settings")
    .await?;
```

---

## 7. 编写业务自定义原生 SQL 迁移

如果你的业务子系统需要独立的实体物理表，可直接在 `migrations/` 目录下追加 SQL 迁移脚本：

```text
my-app/
└── migrations/
    ├── postgres/
    │   └── 001_create_orders.sql
    └── mysql/
        └── 001_create_orders.sql
```

系统会根据当前激活的数据库引擎自动执行对应方言目录下的迁移。

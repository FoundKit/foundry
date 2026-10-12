---
title: 数据库与存储引擎开发指南
description: Foundry 多数据库 SPI 架构、PostgreSQL、MySQL、MariaDB 与 MongoDB 配置、Zero-DDL 动态模型引擎与容器化快速起步。
---

# 数据库与存储引擎开发指南

> [!TIP]
> **完整系统开发手册**：若需查阅 ThinkPHP 风格的链式 ORM 查询构造器 (`OrmSelect`, `OrmInsert`, `OrmUpdate`, `OrmDelete`)、Code-First 实体声明与全流程实战，请参阅 [系统开发完全指南](./system-development/)。

Foundry 采用现代化的 **多数据库 SPI (Service Provider Interface) 插件化存储架构**。业务层、领域逻辑服务、AutoCRUD 引擎与 Axum 处理器统一依赖 `Database` 门面与标准 Store Traits，**彻底实现底层网络驱动零泄露**。

Foundry 官方原生支持四大企业级主流数据库：
1. **PostgreSQL (14+)** - 关系型引擎，支持原生 JSONB 存储与 GIN 倒排索引加速。
2. **MySQL (8.0+)** - 通用单表引擎，具备覆盖索引与延迟关联 (Deferred Join) 深度分页优化。
3. **MariaDB (10.5+)** - 针对 MariaDB 方言特化优化，专属连接协议与高效 InnoDB 覆盖索引。
4. **MongoDB (6.0+)** - 原生 BSON 文档引擎，通过原子 `_counters` 序列提供 `i64` 主键兼容，结合 Wildcard 全量动态索引加速。

*(注：经架构评审，Oracle DB 鉴于其过重的运行时依赖、繁琐的 C 动态库前置条件以及开源社区维护活跃度不足，已正式从路线图中移除。)*

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
        Registry -->|"mysql://"| MySqlEngine["MySqlProvider (通用单表 + 覆盖索引 + 延迟关联)"]
        Registry -->|"mariadb://"| MariaEngine["MariaDbProvider (专属方言 + 覆盖索引)"]
        Registry -->|"mongodb:// / mongodb+srv://"| MongoEngine["MongoProvider (原生 BSON + 原子计数器 + Wildcard 索引)"]
    end
```

### 核心特性
1. **面向方法调用，驱动零泄露**：上层业务代码无需关心底层是 PostgreSQL、MySQL、MariaDB 还是 MongoDB，统一通过 `state.db.records().create(...)`、`state.db.records().list(...)` 等 Trait 方法访问数据。无任何底层连接池类型泄露。
2. **智能协议自适应**：系统启动时自动分析 `DATABASE_URL` 的协议头并激活对应驱动：
   - `postgres://` 或 `postgresql://` $\rightarrow$ 自动激活 PostgreSQL
   - `mysql://` $\rightarrow$ 自动激活 MySQL
   - `mariadb://` $\rightarrow$ 自动激活 MariaDB
   - `mongodb://` 或 `mongodb+srv://` $\rightarrow$ 自动激活 MongoDB
   - 亦可通过环境变量 `DATABASE_TYPE=postgres|mysql|mariadb|mongodb` 显式强制指定。
3. **Cargo Feature 按需编译**：底层驱动通过 Feature Flags 控制，避免不必要的依赖编译膨胀与构建耗时。

---

## 2. 环境配置与 Feature Flags

### Cargo.toml 依赖配置

Foundry 默认开启 `postgres` 特性。根据你选用的底层数据库，在应用 `Cargo.toml` 中开启对应特性：

```toml
[dependencies]
# 1. 仅使用 PostgreSQL (默认)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# 2. 开启 MySQL 支持
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mysql"] }

# 3. 开启 MariaDB 支持
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mariadb"] }

# 4. 开启 MongoDB 支持
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mongodb"] }

# 5. 同时开启全部四种数据库引擎
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["all-databases"] }
```

### 环境变量配置 (`.env`)

#### 使用 PostgreSQL：
```bash
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry_dev
DATABASE_TYPE=postgres
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### 使用 MySQL：
```bash
DATABASE_URL=mysql://root:root@localhost:3306/foundry_dev
DATABASE_TYPE=mysql
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### 使用 MariaDB：
```bash
DATABASE_URL=mariadb://root:root@127.0.0.1:3307/foundry_dev
DATABASE_TYPE=mariadb
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### 使用 MongoDB：
```bash
DATABASE_URL=mongodb://127.0.0.1:27017/foundry_dev
DATABASE_TYPE=mongodb
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

> **提示**：当 `AUTO_MIGRATE=true` 时，服务启动时会自动检测数据库是否存在；若不存在会自动建库（SQL 引擎自动建库与执行表结构迁移；MongoDB 自动创建集合与索引），并初始化超级管理员账号。

---

## 3. 容器化本地开发环境 (Docker / Podman)

通过 `foundry new` 脚手架创建的项目预置了容器化配置文件，你也可以直接通过 Docker 单独启动任意数据库：

```bash
# PostgreSQL 16
docker run -d --name foundry-postgres -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry_dev postgres:16-alpine

# MySQL 8.4
docker run -d --name foundry-mysql -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry_dev mysql:8.4

# MariaDB 11.4
docker run -d --name foundry-mariadb -p 3307:3306 \
  -e MARIADB_ROOT_PASSWORD=root -e MARIADB_DATABASE=foundry_dev mariadb:11.4

# MongoDB 7.0
docker run -d --name foundry-mongodb -p 27017:27017 \
  mongo:7.0

# Redis 8.0
docker run -d --name foundry-redis -p 6379:6379 redis:alpine
```

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

## 5. 存储引擎架构与深度调优

### 5.1 PostgreSQL 引擎
- **存储方案**：标准关系表 + 原生 `JSONB` 格式。
- **倒排索引**：通过 `CREATE INDEX ... USING gin (data)` 建立 GIN 倒排索引，使得对任意深度嵌套的 JSON 属性查询达到毫秒级响应。
- **原子性保障**：利用 `INSERT ... RETURNING *` 原生语法，单次网络往返即可完成记录插入并提取完整字段。

### 5.2 MySQL 通用单表引擎
- **覆盖索引设计**：
  ```sql
  INDEX idx_model_query (model_id, deleted_at, created_at DESC, id DESC),
  INDEX idx_system_query (system_id, deleted_at)
  ```
- **延迟关联 (Deferred Join) 优化**：在百万级深度分页时（如 `OFFSET 100000 LIMIT 20`），先仅扫描索引获取目标主键 ID，再回表获取完整的 JSON 数据，从根本上解决扫描大 JSON 字段带来的 IO 瓶颈。
- **原子事务重查**：通过事务内部原子组合 `INSERT` $\rightarrow$ `last_insert_id()` $\rightarrow$ `SELECT`，统一对外提供强一致返回。

### 5.3 MariaDB 专属引擎
- **协议自适应**：支持 `mariadb://` 专属连接串，并在底层进行自动参数兼容与规范化。
- **方言脚本**：提供针对 MariaDB 10.5+ 的特化初始化脚本，采用高效 UTF8MB4 字符集与高并发 InnoDB 配置。
- **性能优化**：继承延迟关联索引架构，充分发挥 MariaDB 查询优化器在单表 JSON 场景下的性能潜力。

### 5.4 MongoDB 原生文档引擎
- **原生文档存储**：直接将动态记录以 BSON 格式持久化在 `model_records` 集合中。
- **原子自增序列保证**：通过独立的 `_counters` 集合与 `findOneAndUpdate` 原语（`$inc`），保证动态记录生成严格单调自增的 64 位整数主键 (`id: i64`)，确保与 SQL 引擎公共契约 100% 保持一致。
- **Wildcard 全量动态索引**：服务初始化时自动在数据属性上创建 Wildcard 索引：
  ```javascript
  db.model_records.createIndex({ "data.$**": 1 })
  ```
  对动态模型中的任意属性（例如 `data.category = "tech"`）过滤查询时，无需手动创建独立索引即可自动享受索引加速。
- **原生管道与过滤器**：将分页参数、软删除 (`deleted_at: null`) 与租户隔离范围直接转换为高效的原生 BSON 查询与投影过滤器。

---

## 6. 读取与注入强类型配置

通过 `db.get_typed_config::<T>(&system_slug, key)` 可以直接将动态配置反序列化为类型安全的结构体：

```rust
#[derive(Debug, serde::Deserialize)]
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

如果你的业务子系统需要独立的物理实体表，可直接在 `migrations/` 目录下追加 SQL 迁移脚本：

```text
my-app/
└── migrations/
    ├── postgres/
    │   └── 001_create_orders.sql
    ├── mysql/
    │   └── 001_create_orders.sql
    └── mariadb/
        └── 001_create_orders.sql
```

系统在执行 `foundry migrate` 或 `AUTO_MIGRATE=true` 时，会自动感知当前激活的数据库引擎，并执行对应方言目录下的迁移脚本。

---
title: 数据库与存储引擎开发指南
description: Foundry 多数据库 SPI 架构、PostgreSQL、MySQL、MariaDB 与 MongoDB 配置、Zero-DDL 动态模型引擎与容器化快速起步。
---

> [!TIP]
> **完整系统开发教程**：若需查阅链式 ORM 查询构造器 (`OrmSelect`, `OrmInsert`, `OrmUpdate`, `OrmDelete`)、Code-First 实体声明与全流程实战，请参阅 [05. 数据库配置与 Code-First ORM](../development/05-database-orm/)。

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

> **提示**：当 `AUTO_MIGRATE=true` 时，服务启动时会自动检测数据库是否存在；若不存在会自动建库（SQL 引擎自动建库与执行表结构迁移；MongoDB 自动创建集合与索引），并初始化超级管理员账号。当配置为 `AUTO_MIGRATE=false` 时，系统严禁在库表不完整的情况下启动，启动引导期间将自动核验基础结构并拦截未完成同步的数据库，提示错误并安全退出（可执行 `foundry migrate --check` 提前检测）。

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

## 7. 专题/子系统分表设计与 DDL 交付规范 (针对 MySQL / MariaDB)

### 7.1 为什么 MySQL / MariaDB 需要独立物理单表？

在面对多业务专题、海量数据吞吐时，不同数据库引擎的物理特性决定了不同的架构策略：

```mermaid
flowchart TD
    DBSelection["数据库选型与存储架构决策"]
    DBSelection --> Mongo["MongoDB"]
    DBSelection --> PG["PostgreSQL"]
    DBSelection --> MySQL["MySQL / MariaDB"]

    Mongo --> MongoStrategy["【纯 0-DDL】<br/>原生 BSON 文档，集合自动按需创建<br/>Wildcard 索引，天然无表模式维护"]
    PG --> PGStrategy["【0-DDL 优选】<br/>原生 JSONB + GIN 倒排索引 + 分区表<br/>单通用大表依然保持极高性能"]
    MySQL --> MySQLStrategy["【专属分表 + 显式 DDL】<br/>无原生 GIN 索引，单表大 JSON 性能衰退<br/>推荐每个子系统独立单表 + 强类型业务 ORM"]
```

* **MongoDB**：天然是文档型数据库（Schemaless），集合在首次插入记录时自动生成，完全不需要任何 DDL SQL。
* **PostgreSQL**：在 Mode A（0-DDL 通用模型）下，依靠二进制 `JSONB`、`GIN` 倒排索引与部分索引，单表海量多专题依然能维持极佳性能。
* **MySQL / MariaDB 的单表痛点**：
  1. MySQL 的 JSON 本质上只作为长文本/BLOB 存储，缺乏真正的 GIN 倒排索引，动态字段过滤成本较高；
  2. 当数百个专题、几千万行数据全塞入一张通用的 `records` 表时，InnoDB 的 B+Tree 索引深度增加、行锁竞争加剧、Buffer Pool 换入换出频繁，极易成为系统级吞吐瓶颈。
  3. **最佳实践**：针对 MySQL / MariaDB，**推荐使用 `foundry system new <slug> --with-table` 为每个业务子系统拆分为独立的物理表**，并生成专用的 DDL SQL 脚本。

### 7.2 开发期自动迁移 vs 生产期 DBA 手动执行

Foundry 针对多环境建立了严格规范的 DDL 交付契约：

1. **开发与测试环境 (完全自动化)**：
   执行 `foundry migrate` 时，系统不仅会自动执行平台基准表，还会**自动递归扫描 `migrations/<dialect>/*.sql`** 中的所有子系统增量 DDL 脚本并按顺序自动执行，无需开发者手动开终端敲 SQL。
2. **线上生产环境 (安全隔离与 DBA 离线执行)**：
   线上生产数据库通常由专门的 DBA 团队管控，应用程序连接账号出于安全规范一般**严格禁止赋予 `DDL`（如 `CREATE TABLE`、`ALTER TABLE`、`DROP TABLE`）权限**。
   脚手架生成的 `migrations/<dialect>/<slug>.sql` 头部带有完整的审计注释与生产部署指南，运维团队可在发版前直接离线审核并在生产库手动执行，完全符合企业级发布流程。

---

## 8. 全系统严格软删除规范 (零物理 DELETE 权限保障)

### 8.1 架构设计原则

数据是企业最宝贵的核心资产。Foundry 遵循**严格的软删除（Soft Delete / 标记删除）**最佳实践：

> [!IMPORTANT]
> **零物理 DELETE 保障**：系统所有功能模块（包括子系统注册、业务数据记录、动态数据模型与字段元数据、系统全局配置、后台管理员账号）的“删除”操作，**一律只更新 `deleted_at = NOW()` 标记**，底层存储引擎严格杜绝任何 `DELETE FROM ...` 物理删除语句！

### 8.2 覆盖范围与字段约定

所有核心物理表与自动生成的子系统业务表均统一包含以下软删除规范：

| 实体模型 | 物理数据表 | 软删除字段与过滤规则 |
|---|---|---|
| **子系统/专题** | `systems` | `deleted_at: Option<DateTime<Utc>>`，查询必须满足 `WHERE deleted_at IS NULL` |
| **系统配置项** | `system_configs` | `deleted_at: Option<DateTime<Utc>>`，配置检索自动过滤软删除记录 |
| **模型与字段元数据** | `models`, `model_fields` | `deleted_at: Option<DateTime<Utc>>`，字段列表读取与类型校验自动排除失效字段 |
| **后台管理员账号** | `admins` | `deleted_at: Option<DateTime<Utc>>`，登录鉴权自动阻断已删除管理员 |
| **动态业务记录** | `model_records` / `record_values` | `deleted_at: Option<DateTime<Utc>>`，分页与多条件检索内置过滤 |
| **子系统专属物理表** | `<slug>_items`（通过 `--with-table` 生成） | 预置 `deleted_at DATETIME(6) NULL`，配套 Service 使用 `set_now("deleted_at")` |

### 8.3 数据库账号最低权限保障 (最小权限原则)

得益于全系统的软删除架构，**即使用户/DBA 在生产环境中收回了数据库账号的物理 `DELETE` 权限（仅赋予 `SELECT, INSERT, UPDATE`），整个 Foundry 系统的所有功能（包括管理员管理、模型维护、配置修改、业务数据增删改查）依然 100% 顺畅正常运行！**

这一特性带来了巨大的安全收益：
1. **防止误删与恶意删库**：即使数据库连接串遭遇极端泄露或发生未预期的代码缺陷，攻击者与应用程序由于账号本身没有物理 `DELETE` 权限，永远无法直接抹除数据库物理行。
2. **便于审计与数据恢复**：任何删除动作均可留痕，且可通过将 `deleted_at` 重置为 `NULL` 实现秒级数据撤销与灾难恢复。

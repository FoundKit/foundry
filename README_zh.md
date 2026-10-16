# Foundry

<div align="center">

**现代、模块化、完全解耦的 Rust 后端平台与应用框架。**

*构建、隔离与扩展独立的后端业务系统和管理控制台，上游框架升级零合并冲突。*

<p align="center">
  <a href="README.md">English</a> | <b>简体中文</b>
</p>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-16%2B%20%2F%2018%2B-336791.svg)](https://www.postgresql.org)
[![MySQL](https://img.shields.io/badge/mysql-8.0%2B%20%2F%208.4%2B-4479A1.svg)](https://www.mysql.com)
[![Redis](https://img.shields.io/badge/redis-7%2B%20%2F%208-dc382d.svg)](https://redis.io)

</div>

---

## 🧭 概述与核心定位

Foundry 是一个专为构建高可靠、多业务系统后端服务及内嵌管理控制台而设计的 **Rust 后端平台与应用框架**。

与传统的脚手架模版（Boilerplate）不同，Foundry 将 **框架/平台（Framework/Platform）** 与 **用户业务应用（User Application）** 彻底分离开来：

```text
Foundry 框架仓库 (Framework)
    │
    ├── crates/foundry             # 顶层门面 Crate (FoundryApp, Builder, prelude)
    ├── crates/foundry_core        # 上下文、错误处理、响应模型、Subsystem trait
    ├── crates/foundry_storage     # Zero-DDL 动态数据模型、系统配置、多数据库 SPI (PostgreSQL, MySQL, MariaDB, MongoDB)、Redis
    ├── crates/foundry_auth        # 管理员身份与认证、Argon2id、JWT、专题级 RBAC
    ├── crates/foundry_engine      # 多系统路由、Auto-CRUD 引擎、审计日志中间件
    ├── crates/foundry_extension   # 生命周期变更钩子 (Mutation Hook) 与扩展流水线
    ├── crates/foundry_cli         # `foundry` 与 `foundry-cli` 开发者命令行工具
    ├── apps/admin                 # 独立解耦的 React + Tailwind 可视化 Admin 控制台外壳
    └── examples/blog_platform     # 独立应用参考范例
            │
            │ cargo publish
            ↓
    Foundry Cargo Crates
            │
            ↓
用户业务应用 (独立 Git 仓库)
    ├── Cargo.toml                 # [dependencies] foundry = "0.1" (自动配置所选数据库特性)
    ├── .env                       # 环境变量配置 (包含 .env.example)
    ├── src/
    │   ├── main.rs                # 应用启动入口 (基于 FoundryApp::builder())
    │   └── systems/               # 用户业务子系统
    │       ├── blog/              # 控制器 + 领域逻辑 + DTO 校验 + 自定义 Admin 页面
    │       └── newsletter/
    └── migrations/                # （可选）按需生成的方言迁移脚本目录 (执行 `foundry ddl` 或 `foundry migrate -g` 时生成)
        └── init.sql               # 平台初始化全量表结构与初始超管账号 (MongoDB 为 init.js)
```

---

## ⚡ 开发者体验

创建并运行一个全新的 Foundry 工程仅需数秒：

```bash
# 1. 安装 Foundry CLI 命令行工具 (直接从 GitHub 仓库安装)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 2. 创建全新的独立应用工程 (默认 PostgreSQL，或使用 --db mysql / mariadb / mongodb)
foundry new my-app --db postgres
cd my-app

# 3. 准备数据库服务 (使用现有数据库，或使用 Docker 启动本地依赖)
# 以 Docker 启动 PostgreSQL 18 与 Redis 为例：
docker run -d --name my-postgres -p 5432:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry postgres:18-alpine
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# 4. 启动后端服务 (AUTO_MIGRATE=true 时自动完成库表初始化与超管注入)
cargo run
```

生成的业务应用完全独立，通过标准 Cargo 依赖引入 Foundry：

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
foundry = "0.1.0"
tokio = { version = "1.44", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[foundry::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FoundryConfig::from_env();

    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(SampleSubsystem)
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

---

## 🌟 核心架构特性

1. **框架与业务完全解耦（Framework & Application Decoupling）**：应用开发者在独立的 Git 仓库中构建业务。升级 Foundry 仅需执行 `cargo update`，彻底告别上游 Git 合并冲突。
2. **顶层门面（`foundry`）与公共 API**：直观优雅的 `FoundryApp::builder()` API、类型安全的错误信封（`AppError`、`AppResult`）、统一规范的响应结构体（`ApiResponse`）以及 `foundry::prelude::*` 丰富重导出。
3. **子系统架构（Subsystem Architecture）**：通过实现 `SubsystemModule` 将业务领域组织为高内聚、自包含的子系统（包含控制器、业务领域服务、数据校验 DTO 与自定义 Admin 管理页面）。
4. **双模存储与多数据库 SPI**：PostgreSQL 与 MongoDB 原生支持极致 Zero-DDL 免表迁移与 Auto-CRUD；MySQL 与 MariaDB 针对无 GIN 倒排索引物理限制，采用专题专属物理分表 DDL 与延迟关联分页，兼顾敏捷迭代与高并发生产性能。全数据库支持零物理 DELETE 权限安全运行。
5. **即刻生成的 RESTful Auto-CRUD**：所有动态数据模型自动获得高性能 REST API 接口（`/api/v1/s/{system_slug}/{model_slug}`），在所有适配的数据库上均原生支持分页与检索。
6. **解耦的 Admin 控制台外壳与 SDK 桥接**：独立 Admin SPA 前端（`apps/admin`）通过沙箱视图内嵌各子系统的自定义页面（HTML、React、Vue 等），并通过 `postMessage` 协议自动注入 JWT 认证凭据与主题配置。
7. **分级权限体系（Hierarchical RBAC）**：支持超级管理员（`super_admin`）、平台管理员（`admin`）和专题专属管理员（`topic_admin`）的细粒度角色权限控制。
8. **生命周期变更钩子（Mutation Hooks）**：通过实现 `MutationHook` 拦截实体数据的完整生命周期事件（`before_create`、`after_create`、`before_update`、`after_update`、`before_delete`、`after_delete`）。

---

## 📦 工作区 Crates

| Crate | 说明 |
|---|---|
| [`foundry`](crates/foundry) | 顶层门面 Crate，提供 `FoundryApp`、`FoundryBuilder` 与核心 `prelude`。 |
| [`foundry_core`](crates/foundry_core) | 核心原语、`SystemContext`、`SubsystemModule`、`AppError` 与统一响应模型。 |
| [`foundry_storage`](crates/foundry_storage) | 多数据库 SPI (PostgreSQL, MySQL, MariaDB, MongoDB)、Redis 缓存、Zero-DDL 动态数据模型、专题独立分表 DDL 与软删除支持。 |
| [`foundry_auth`](crates/foundry_auth) | 管理员身份认证、Argon2id 密码哈希、JWT Token 服务与专题级 RBAC 守卫。 |
| [`foundry_engine`](crates/foundry_engine) | 统一 Axum 路由分发、Auto-CRUD 处理引擎与操作审计日志中间件。 |
| [`foundry_extension`](crates/foundry_extension) | 实体生命周期变更钩子（Mutation Hook）流水线与扩展机制。 |
| [`foundry_cli`](crates/foundry_cli) | 开发者命令行工具（`foundry` 与 `foundry-cli`），用于项目创建与子系统脚手架生成。 |

---

## 🛠️ CLI 命令与核心工作流

Foundry 提供了精简高效的开发者命令行工具（`foundry` / `foundry-cli`），用于管理应用工程、业务子系统和数据模型。

### 1. 数据库配置与多引擎 CLI 支持
 
Foundry 应用统一通过 `.env` 中的 `DATABASE_URL` 与 `DATABASE_TYPE` 连接底层数据库。可以在创建应用时直接指定数据库，或随时按需生成方言 DDL：

```bash
# 脚手架指定数据库（纯净代码骨架，migrations 目录按需生成）：
foundry new my-app --db postgres    # PostgreSQL (默认)
foundry new my-app --db mysql       # MySQL 8.0+
foundry new my-app --db mariadb     # MariaDB 10.5+
foundry new my-app --db mongodb     # MongoDB 6.0+

# 随时生成指定数据库方言的 DDL 迁移脚本（自动创建 migrations/ 目录）：
foundry ddl --db postgres           # 生成 PostgreSQL DDL 到 migrations/init.sql
foundry ddl --db mysql              # 生成 MySQL DDL 到 migrations/init.sql
foundry ddl --db mongodb            # 生成 MongoDB init.js 到 migrations/init.js

# 手动执行数据库迁移（或使用 --generate 生成迁移文件）：
foundry migrate
foundry migrate --generate          # 为当前项目生成 migrations/ 目录与方言脚本
```

* **全自动初始化 (`AUTO_MIGRATE=true`)**：运行 `cargo run` 启动时，框架自动检测并完成数据库表结构/集合、覆盖索引创建与初始超管账号（`admin` / `admin123456`）注入。
* **DBA 严格审计管控 (`AUTO_MIGRATE=false`)**：可由 DBA 在部署前手动执行 `migrations/` 下的原生脚本（`psql -f migrations/init.sql`、`mysql < migrations/init.sql`、`mongosh migrations/init.js` 等）。

### 2. 子系统（子项目）操作

将业务能力组织到高内聚、相互隔离的子系统中：

```bash
# 1. 在 src/systems/<slug>/ 下创建代码优先的业务子系统
foundry system new billing --name "Billing Center"

# 2. 在 src/systems/mod.rs 中注册声明：
#    pub mod billing;
#    pub use billing::BillingSubsystem;

# 3. 在 src/main.rs 中注册子系统：
#    .register_subsystem(BillingSubsystem)

# 4. 列出所有检测到的子系统
foundry system list
```

### 3. 数据模型创建与存储

Foundry 支持 **ORM 优先 (ORM-First, Prisma 风格)** 的现代化开发架构，并提供灵活的多范式存储选择：

#### 方式 A：Code-First ORM 实体模型（Prisma 风格，推荐强类型）
无需手写 SQL 方言迁移。在业务子系统中直接通过 Rust 代码定义实体结构（类似 Prisma Schema）。执行 `foundry migrate` 或 `foundry ddl` 时，Foundry 会自动将代码中的 ORM 模型编译为对应数据库方言（PostgreSQL, MySQL, MariaDB）的 DDL 结构并应用：

```rust
use foundry::prelude::*;

impl SubsystemModule for BillingSubsystem {
    fn slug(&self) -> &'static str { "billing" }
    fn display_name(&self) -> &'static str { "账单与支付中心" }

    // 代码定义 ORM 模型（Prisma 风格）
    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            EntitySchema::new("billing_invoices")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("invoice_no", 64).not_null().unique())
                .column(ColumnDef::decimal("amount", 10, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null())
                .column(ColumnDef::json("metadata").default_empty_json())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("updated_at").default_current_timestamp())
                .index(IndexDef::new("idx_invoices_sys", &["system_id"]))
        ]
    }
}
```

CRUD 增删改查同样使用类型安全的 ORM 构建器（或仓储层封装方法），底层自动适配各方言参数占位符（`$1` vs `?`）：
```rust
use foundry::prelude::*;

// ORM 插入
let (sql, binds) = OrmInsert::into_table("billing_invoices")
    .value("system_id", "billing")
    .value("invoice_no", "INV-2026-001")
    .value("amount", 99.50)
    .to_sql(Dialect::Postgres);

// ORM 分页查询
let (sql, binds) = OrmSelect::from_table("billing_invoices")
    .where_eq("system_id", "billing")
    .order_by_desc("created_at")
    .limit(20)
    .offset(0)
    .to_sql(Dialect::Postgres);
```

#### 方式 B：Zero-DDL 动态数据模型（免表迁移，即刻生成 RESTful Auto-CRUD）
无需手写任何建表脚本。通过可视化 Admin 控制台（`/admin`）动态定义模型，或在业务代码中直接操作记录：

```rust
use foundry_storage::models::RecordStore;
use serde_json::json;

// 创建动态记录（严格按 system_slug 与 model_slug 进行隔离）
let record = RecordStore::create(
    &db,
    &ctx.system_slug,  // 子系统标识 (subsystem slug)
    "articles",         // 模型标识 (model slug)
    json!({ "title": "First Post", "views": 10 })
).await?;

// 按 ID 查询或获取分页列表
let item = RecordStore::get_by_id(&db, &ctx.system_slug, "articles", record.id).await?;
let list = RecordStore::list(&db, &ctx.system_slug, "articles", query).await?;
```
所有动态数据模型将自动挂载 REST API 接入点：`/api/v1/s/{system_slug}/{model_slug}`。

#### 方式 C：原生 SQL / 脚本逃生通道（特殊触发器、MongoDB 脚本）
对于无关系型 ORM 的数据库（如 MongoDB）或特殊复杂场景，可通过 `SubsystemModule::migration_ddl` 或 `migrations/` 目录添加自定义原生 SQL 或脚本，同样受 `foundry migrate` 统一管控。

### 4. 项目结构与子系统校验

```bash
# 校验项目结构与子系统完整性
foundry validate
```

> **管理员与身份认证（IAM）管理**：管理员账号、角色授权以及密码管理均在内嵌的 React Admin 可视化控制台（`/admin/`）中完成，使 CLI 工具专注于开发体验与代码生命周期管理。

---

## 📚 示例应用

查看 [`examples/blog_platform`](examples/blog_platform) 获取完整的参考应用范例，其中演示了：
- 使用 `FoundryApp::builder()` 启动引导应用程序
- 多自定义业务子系统协同（`blog` 与 `newsletter`）
- 子系统自定义控制器、业务领域逻辑服务与参数校验 DTO
- 自定义 Admin 控制台页面集成（Admin UI Studio）
- 自定义数据生命周期变更钩子（`BlogMutationHook`）
- 完善的端到端集成测试（`tests/integration_test.rs`）

---

## 📖 文档指南

- 🌐 [官方文档站点](https://foundkit.github.io/foundry/zh/)
- 📘 [快速入门指南 (5 分钟极简起步)](docs/src/content/docs/zh/getting-started.md)
- 🚀 [系统开发核心教程 (模块化开发手册)](docs/src/content/docs/zh/development/index.md)
- 🏗️ [架构设计蓝图](docs/src/content/docs/zh/architecture/blueprint.md)
- 💾 [数据库配置与 Code-First ORM](docs/src/content/docs/zh/development/05-database-orm.md)
- ✉️ [邮件服务与验证码中心](docs/src/content/docs/zh/guides/email-verification.md)
- 🔌 [动态外部子系统扩展](docs/src/content/docs/zh/guides/external-subsystems.md)
- 🗺️ [项目开发路线图](docs/src/content/docs/zh/roadmap.md)

---

## 📄 开源许可协议

本项目采用双重许可协议，你可根据需要选择遵循 [Apache License 2.0](LICENSE-APACHE) 或 [MIT License](LICENSE-MIT)。

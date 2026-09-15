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
    ├── crates/foundry_storage     # Zero-DDL 动态数据模型、系统配置、PostgreSQL、Redis
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
    ├── Cargo.toml                 # [dependencies] foundry = "0.1"
    ├── dev/                       # 本地开发专用资源 (.gitignore 忽略)
    │   ├── docker-compose.yml     # PostgreSQL 18 + Redis 7 本地容器编排 (挂载 init.sql)
    │   ├── init-db.sh             # 本地开发数据库初始化/重置脚本
    │   └── init-dev-db.sql        # 独立开发数据库创建引导脚本
    ├── src/
    │   ├── main.rs                # 应用启动入口 (基于 FoundryApp::builder())
    │   └── systems/               # 用户业务子系统
    │       ├── blog/              # 控制器 + 领域逻辑 + DTO 校验 + 自定义 Admin 页面
    │       └── newsletter/
    ├── config/                    # 应用配置
    └── migrations/                # 业务专属数据库迁移脚本
        └── init.sql               # 平台初始化全量表结构与初始超管账号 Seed
```

---

## ⚡ 开发者体验

创建并运行一个全新的 Foundry 工程仅需数秒：

```bash
# 1. 安装 Foundry CLI 命令行工具 (直接从 GitHub 仓库安装)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 2. 创建全新的独立应用工程
foundry new my-app
cd my-app

# 3. 启动本地开发数据库 (PostgreSQL 18 + Redis 7)
docker compose -f dev/docker-compose.yml up -d

# 4. 启动后端服务
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
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
```

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
4. **Zero-DDL 动态存储引擎**：在可视化 Admin 控制台中直接配置系统参数与动态数据模型，无需编写原生 SQL DDL 迁移脚本，无锁表停机烦恼。
5. **即刻生成的 RESTful Auto-CRUD**：所有动态数据模型自动获得高性能 REST API 接口（`/api/v1/s/{system_slug}/{model_slug}`），原生支持分页与 GIN JSON 检索。
6. **解耦的 Admin 控制台外壳与 SDK 桥接**：独立 Admin SPA 前端（`apps/admin`）通过沙箱视图内嵌各子系统的自定义页面（HTML、React、Vue 等），并通过 `postMessage` 协议自动注入 JWT 认证凭据与主题配置。
7. **分级权限体系（Hierarchical RBAC）**：支持超级管理员（`super_admin`）、平台管理员（`admin`）和专题专属管理员（`topic_admin`）的细粒度角色权限控制。
8. **生命周期变更钩子（Mutation Hooks）**：通过实现 `MutationHook` 拦截实体数据的完整生命周期事件（`before_create`、`after_create`、`before_update`、`after_update`、`before_delete`、`after_delete`）。

---

## 📦 工作区 Crates

| Crate | 说明 |
|---|---|
| [`foundry`](crates/foundry) | 顶层门面 Crate，提供 `FoundryApp`、`FoundryBuilder` 与核心 `prelude`。 |
| [`foundry_core`](crates/foundry_core) | 核心原语、`SystemContext`、`SubsystemModule`、`AppError` 与统一响应模型。 |
| [`foundry_storage`](crates/foundry_storage) | PostgreSQL 连接池、Redis 缓存、Zero-DDL 动态数据模型与数据库迁移支持。 |
| [`foundry_auth`](crates/foundry_auth) | 管理员身份认证、Argon2id 密码哈希、JWT Token 服务与专题级 RBAC 守卫。 |
| [`foundry_engine`](crates/foundry_engine) | 统一 Axum 路由分发、Auto-CRUD 处理引擎与操作审计日志中间件。 |
| [`foundry_extension`](crates/foundry_extension) | 实体生命周期变更钩子（Mutation Hook）流水线与扩展机制。 |
| [`foundry_cli`](crates/foundry_cli) | 开发者命令行工具（`foundry` 与 `foundry-cli`），用于项目创建与子系统脚手架生成。 |

---

## 🛠️ CLI 命令与核心工作流

Foundry 提供了精简高效的开发者命令行工具（`foundry` / `foundry-cli`），用于管理应用工程、业务子系统和数据模型。

### 1. 本地开发数据库

生成的工程包含专门的 `dev/docker-compose.yml`（已被 `.gitignore` 忽略以保持仓库整洁），默认配置挂载 `migrations/init.sql`，首次启动时自动完成数据库初始化：

```bash
# 启动本地 PostgreSQL 18 与 Redis 7（首次启动会自动执行 migrations/init.sql）
docker compose -f dev/docker-compose.yml up -d

# 查看服务运行状态
docker compose -f dev/docker-compose.yml ps

# 手动重新应用或重置本地开发数据库
bash dev/init-db.sh

# 停止本地数据库
docker compose -f dev/docker-compose.yml down

# 重置本地数据库及其数据卷（全新重新初始化）
docker compose -f dev/docker-compose.yml down -v
```

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

支持在 Zero-DDL 动态模型与强类型 SQLx Schema 之间灵活选择：

#### 方式 A：Zero-DDL 动态数据模型（即刻生成 RESTful Auto-CRUD）
无需编写任何 SQL DDL 迁移。通过可视化 Admin 控制台（`/admin`）动态定义模型，或在业务代码中直接操作记录：

```rust
use foundry_storage::models::RecordStore;
use serde_json::json;

// 创建动态记录（严格按 system_slug 与 model_slug 进行租户隔离）
let record = RecordStore::create(
    &db,
    &ctx.system_slug,  // 子系统标识 (subsystem slug)
    "articles",         // 模型标识 (model slug)
    json!({ "title": "First Post", "views": 10 })
).await?;

// 按 ID 查询或获取分页列表
let item = RecordStore::get_by_id(&db, &ctx.system_slug, "articles", record.id).await?;
let list = RecordStore::list(&db, &ctx.system_slug, "articles", 1, 20).await?;
```
所有动态数据模型将自动挂载 REST API 接入点：`/api/v1/s/{system_slug}/{model_slug}`。

#### 方式 B：原生 SQL 迁移与强类型模型（高并发与严苛事务场景）
1. 在 `migrations/001_create_orders.sql` 中新增迁移脚本：
   ```sql
   CREATE TABLE IF NOT EXISTS orders (
       id BIGSERIAL PRIMARY KEY,
       system_slug VARCHAR(64) NOT NULL,
       order_no VARCHAR(64) NOT NULL UNIQUE,
       amount BIGINT NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   CREATE INDEX IF NOT EXISTS idx_orders_system_slug ON orders(system_slug);
   ```
2. 执行迁移：
   ```bash
   foundry migrate
   # 或者在 .env 中设置 AUTO_MIGRATE=true，服务启动时将自动应用
   ```
3. 使用 SQLx 配合 `#[derive(sqlx::FromRow)]` 进行强类型查询。

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
- 📘 [快速入门指南](docs/src/content/docs/zh/getting-started.md)
- 🏗️ [架构设计蓝图](docs/src/content/docs/zh/architecture/blueprint.md)
- 🔌 [子系统与扩展开发指南](docs/src/content/docs/zh/guides/extensions.md)
- 🗺️ [项目开发路线图](docs/src/content/docs/zh/roadmap.md)

---

## 📄 开源许可协议

本项目采用双重许可协议，你可根据需要选择遵循 [Apache License 2.0](LICENSE-APACHE) 或 [MIT License](LICENSE-MIT)。

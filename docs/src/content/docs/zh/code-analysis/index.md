---
title: "源码学习指南与全景导读"
description: "Foundry 框架系统性学习路径、架构概览、数据流与请求生命周期全景图。"
---


> **📚 阶段源码深度剖析目录**：
> 1. [阶段 1：foundry_core 核心类型与接口定义](./01-core/)
> 2. [阶段 2：foundry_storage 存储层实现与动态模型](./02-storage/)
> 3. [阶段 3：foundry_auth 认证与授权系统](./03-auth/)
> 4. [阶段 4：foundry_extension 扩展系统与生命周期钩子](./04-extension/)
> 5. [阶段 5：foundry_engine 请求引擎与自动 CRUD](./05-engine/)
> 6. [阶段 6：foundry 门面层与应用组装](./06-foundry/)
> 7. [阶段 7：Blog Platform 完整业务实战示例](./07-blog-platform/)
> 8. [阶段 8：apps/admin 前端后台深度剖析](./08-admin-frontend/)

# Foundry 项目学习指南

> 完全掌控 Foundry 框架的系统性学习路径
>
> 最后更新：2026-09-15

---

## 📋 目录

1. [项目概览](#1-项目概览)
2. [核心架构理解](#2-核心架构理解)
3. [Crate 详细解析](#3-crate-详细解析)
4. [数据流与请求生命周期](#4-数据流与请求生命周期)
5. [学习路径建议](#5-学习路径建议)
6. [实践项目](#6-实践项目)
7. [常见问题与陷阱](#7-常见问题与陷阱)

---

## 1. 项目概览

### 1.1 什么是 Foundry？

Foundry 是一个**现代化、模块化、解耦的 Rust 后端平台与框架**，专为构建多系统后端服务与内置管理控制面板而设计。

**核心定位差异：**
```
传统框架模式：框架 = 模板（Fork 后修改）
Foundry 模式：框架 = 独立 Crate 依赖（用户项目完全独立）
```

### 1.2 技术栈

| 层级 | 技术选择 | 用途 |
|------|---------|------|
| **语言** | Rust 2024 Edition (1.85+) | 高性能、类型安全 |
| **异步运行时** | Tokio 1.44 | 异步 I/O |
| **Web 框架** | Axum 0.8 | HTTP 路由与处理 |
| **数据库** | PostgreSQL 16-18 | 主数据存储 |
| **缓存** | Redis 7-8 | 会话与缓存 |
| **前端** | React 18 + TailwindCSS | 管理后台 |
| **身份认证** | Argon2id + JWT | 安全认证 |

### 1.3 项目结构总览

```
foundry/
├── crates/                    # 核心框架 Crates（7 个模块）
│   ├── foundry               # [门面] 统一入口与 Prelude
│   ├── foundry_core          # [核心] 上下文、错误、响应、Subsystem trait
│   ├── foundry_storage       # [存储] 数据库、Redis、动态模型
│   ├── foundry_auth          # [认证] 管理员、JWT、RBAC
│   ├── foundry_engine        # [引擎] 路由、Auto-CRUD、审计
│   ├── foundry_extension     # [扩展] 生命周期钩子
│   └── foundry_cli           # [CLI] 项目脚手架工具
├── apps/
│   ├── admin/                # [前端] React 管理后台（独立 SPA）
│   └── server/               # [示例] 参考服务器实现
└── examples/
    └── blog_platform/        # [实战] 博客系统完整示例
```

---

## 2. 核心架构理解

### 2.1 分层架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    用户应用层 (User App)                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  main.rs: FoundryApp::builder()                        │ │
│  │    ↓                                                    │ │
│  │  systems/blog/      (BlogSubsystem)                    │ │
│  │  systems/newsletter/(NewsletterSubsystem)              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                           ↓ 依赖
┌─────────────────────────────────────────────────────────────┐
│               Foundry 框架层 (Framework Crates)              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  foundry (Facade)                                      │ │
│  │    ├── FoundryApp                                      │ │
│  │    ├── FoundryBuilder                                  │ │
│  │    └── prelude (统一导出)                              │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌──────────────┬──────────────┬──────────────┬──────────┐ │
│  │ foundry_core │ foundry_     │ foundry_     │ foundry_ │ │
│  │              │ storage      │ auth         │ engine   │ │
│  │ - Context    │ - DB Pool    │ - JWT        │ - Router │ │
│  │ - Error      │ - Redis      │ - Argon2id   │ - CRUD   │ │
│  │ - Subsystem  │ - Models     │ - RBAC       │ - Audit  │ │
│  └──────────────┴──────────────┴──────────────┴──────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ foundry_extension (Hooks)                              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                           ↓ 数据访问
┌─────────────────────────────────────────────────────────────┐
│              基础设施层 (Infrastructure)                      │
│  ┌──────────────────────┬──────────────────────────────────┐│
│  │  PostgreSQL 18       │  Redis 7                         ││
│  │  (主数据库 + 元数据)  │  (会话 + 缓存)                    ││
│  └──────────────────────┴──────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心设计理念

#### A. Subsystem（子系统）架构

每个业务域独立封装为一个 `SubsystemModule`：

```rust
pub trait SubsystemModule: Send + Sync {
    fn slug(&self) -> &str;           // 唯一标识（如 "blog"）
    fn name(&self) -> &str;           // 显示名称
    fn description(&self) -> &str;    // 描述
    
    // 可选：自定义路由
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        router
    }
    
    // 可选：自定义管理后台页面
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![]
    }
}
```

**优势：**
- 高内聚：一个子系统包含 Controllers、Services、DTOs、Admin Views
- 低耦合：子系统之间互不干扰
- 可测试：每个子系统可独立测试

#### B. Zero-DDL 动态模型

通过管理后台或 API 动态定义数据模型，无需编写 SQL DDL：

```rust
// 创建动态记录（自动生成 CRUD API）
let record = RecordStore::create(
    &db,
    "blog",          // system_slug
    "articles",      // model_slug
    json!({
        "title": "Hello World",
        "content": "...",
        "views": 0
    })
).await?;

// 自动获得 RESTful 端点：
// POST   /api/v1/s/blog/articles
// GET    /api/v1/s/blog/articles
// GET    /api/v1/s/blog/articles/:id
// PUT    /api/v1/s/blog/articles/:id
// DELETE /api/v1/s/blog/articles/:id
```

#### C. 生命周期钩子（Mutation Hooks）

拦截实体生命周期事件：

```rust
#[async_trait]
impl MutationHook for BlogMutationHook {
    async fn after_create(&self, ctx: &HookContext) -> Result<(), HookError> {
        // 文章创建后发送通知
        if ctx.system_slug == "blog" && ctx.model_slug == "articles" {
            send_notification(&ctx.record).await?;
        }
        Ok(())
    }
}
```

---

## 3. Crate 详细解析

### 3.1 `foundry` - 门面 Crate

**职责：** 提供统一的高层 API 入口点

**核心导出：**
```rust
pub use app::{FoundryApp, FoundryBuilder, FoundryConfig};
pub mod prelude;  // 包含所有常用类型
```

**学习重点：**
- `FoundryApp::builder()` 构建器模式
- `prelude` 模块导出内容（查看 `src/prelude.rs`）

**关键文件：**
- `src/app.rs` - 应用启动逻辑
- `src/prelude.rs` - 便捷导入

---

### 3.2 `foundry_core` - 核心原语

**职责：** 定义核心类型、错误处理、上下文、Subsystem trait

**核心模块：**

| 文件 | 职责 |
|------|------|
| `context.rs` | `SystemContext` - 请求上下文传递 |
| `error.rs` | `AppError`, `AppResult`, `ErrorEnvelope` - 统一错误处理 |
| `response.rs` | `ApiResponse`, `PaginatedData` - 标准响应格式 |
| `subsystem.rs` | `SubsystemModule` trait 定义 |
| `types.rs` | 通用类型（`FieldType`, `SystemStatus` 等） |

**学习路径：**
1. **先读** `error.rs` - 理解错误传播机制
2. **后读** `subsystem.rs` - 理解子系统接口
3. **再读** `context.rs` - 理解请求上下文
4. **最后** `response.rs` - 理解响应格式

**关键概念：**

```rust
// 统一错误类型
pub enum AppError {
    NotFound(String),
    Unauthorized,
    ValidationError(String),
    StorageError(String),
    // ...
}

// 统一结果类型
pub type AppResult<T> = Result<T, AppError>;

// 系统上下文
pub struct SystemContext {
    pub system_slug: String,
    pub admin_id: Option<Uuid>,
    pub request_id: String,
}
```

---

### 3.3 `foundry_storage` - 存储层

**职责：** 数据库连接、Redis 缓存、动态模型 CRUD

**核心文件：**

| 文件 | 职责 |
|------|------|
| `db.rs` | SQLx PostgreSQL 连接池管理 |
| `redis_client.rs` | Redis 客户端封装 |
| `models.rs` | `RecordStore` - Zero-DDL 动态模型 |
| `systems.rs` | `SystemStore` - 子系统元数据管理 |
| `admins.rs` | `AdminStore` - 管理员账户 CRUD |
| `configs.rs` | `ConfigStore` - 系统配置 KV 存储 |
| `audit.rs` | `AuditStore` - 审计日志 |
| `entities.rs` | 数据库实体定义（Models） |

**学习路径：**
1. **先读** `db.rs` - 理解连接池初始化
2. **重点读** `models.rs` - 理解动态模型实现（核心特性）
3. **后读** `systems.rs` - 理解子系统如何持久化
4. **选读** `admins.rs`、`configs.rs` - 理解其他存储抽象

**核心 API：**

```rust
// 动态模型 CRUD
RecordStore::create(db, system_slug, model_slug, data) -> Record
RecordStore::get_by_id(db, system_slug, model_slug, id) -> Record
RecordStore::list(db, system_slug, model_slug, page, size) -> Vec<Record>
RecordStore::update(db, system_slug, model_slug, id, data) -> Record
RecordStore::delete(db, system_slug, model_slug, id) -> ()

// 配置 KV
ConfigStore::set(db, system_slug, key, value) -> ()
ConfigStore::get(db, system_slug, key) -> Option<String>
```

---

### 3.4 `foundry_auth` - 认证与授权

**职责：** 管理员身份认证、JWT、RBAC 权限控制

**核心文件：**

| 文件 | 职责 |
|------|------|
| `jwt.rs` | `JwtService` - JWT 签发与验证 |
| `password.rs` | `PasswordService` - Argon2id 密码哈希 |
| `rbac.rs` | `RbacGuard` - 基于角色的访问控制 |

**角色体系：**

```rust
pub enum AdminRole {
    SuperAdmin,      // 超级管理员（全局权限）
    Admin,           // 平台管理员
    TopicAdmin,      // 主题管理员（限定子系统）
}
```

**学习路径：**
1. **先读** `password.rs` - 理解密码哈希验证
2. **后读** `jwt.rs` - 理解 Token 生成与解析
3. **最后** `rbac.rs` - 理解权限守卫实现

**核心 API：**

```rust
// 密码哈希
PasswordService::hash(password) -> String
PasswordService::verify(password, hash) -> bool

// JWT
JwtService::create_token(admin_id, role) -> String
JwtService::verify_token(token) -> Claims

// RBAC 中间件
require_auth(ExtractAdminClaims)
require_super_admin()
```

---

### 3.5 `foundry_engine` - 请求引擎

**职责：** Axum 路由注册、Auto-CRUD 处理器、审计中间件

**核心文件：**

| 文件/目录 | 职责 |
|----------|------|
| `router.rs` | 统一路由注册（Admin API + Subsystem API） |
| `state.rs` | `AppState` - 应用全局状态 |
| `handlers/` | CRUD 处理器（系统、模型、配置、管理员等） |
| `middleware/` | 审计日志、错误处理中间件 |
| `external.rs` | 静态资源嵌入（Admin 前端） |

**学习路径：**
1. **先读** `state.rs` - 理解应用状态管理
2. **重点读** `router.rs` - 理解路由如何组装
3. **深入** `handlers/models.rs` - 理解 Auto-CRUD 实现
4. **选读** `middleware/audit.rs` - 理解审计日志拦截

**路由结构：**

```
/api/v1
  ├── /auth                   # 认证端点
  ├── /systems                # 子系统管理
  ├── /admins                 # 管理员管理
  ├── /configs                # 配置管理
  ├── /audit-logs             # 审计日志
  └── /s/:system_slug         # Subsystem 动态路由
      ├── /:model_slug        # Auto-CRUD 端点
      └── /custom/*           # 自定义子系统路由

/admin                        # React 前端
/__health                     # 健康检查
```

---

### 3.6 `foundry_extension` - 扩展系统

**职责：** 提供生命周期钩子机制

**核心文件：**
- `hooks.rs` - `MutationHook` trait 定义

**钩子事件：**

```rust
#[async_trait]
pub trait MutationHook: Send + Sync {
    async fn before_create(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
    async fn after_create(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
    async fn before_update(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
    async fn after_update(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
    async fn before_delete(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
    async fn after_delete(&self, ctx: &HookContext) -> Result<(), HookError> {
        Ok(())
    }
}
```

**使用场景：**
- 数据验证（before_create）
- 发送通知（after_create）
- 清理关联数据（before_delete）
- 更新缓存（after_update）

---

### 3.7 `foundry_cli` - 命令行工具

**职责：** 项目脚手架、子系统生成、数据库迁移

**核心命令：**

```bash
foundry new <name>          # 创建新项目
foundry system new <slug>   # 创建新子系统
foundry system list         # 列出所有子系统
foundry migrate             # 执行数据库迁移
foundry validate            # 验证项目结构
```

**学习路径：**
- 查看 `src/bin/` 下的命令实现
- 理解模板生成逻辑

---

## 4. 数据流与请求生命周期

### 4.1 启动流程

```rust
// 1. 加载配置
let config = FoundryConfig::from_env();

// 2. 构建应用
let app = FoundryApp::builder()
    .config(config)
    .register_subsystem(BlogSubsystem)     // 注册子系统
    .register_hook(BlogMutationHook)       // 注册钩子
    .build()                                // 初始化（连接数据库、Redis）
    .await?;

// 3. 启动服务器
app.run().await?;  // 绑定端口，开始监听
```

**内部步骤：**
1. 连接 PostgreSQL（`foundry_storage::db::init_pool`）
2. 连接 Redis（`foundry_storage::redis_client::init`）
3. 加载已注册子系统元数据
4. 构建 Axum 路由树（`foundry_engine::router::build_router`）
5. 应用中间件（审计、CORS、超时）
6. 绑定监听地址并启动

---

### 4.2 请求处理流程（Auto-CRUD 示例）

**场景：** `POST /api/v1/s/blog/articles` 创建文章

```
┌───────────────────────────────────────────────────────────────┐
│ 1. HTTP 请求到达                                               │
│    POST /api/v1/s/blog/articles                               │
│    Authorization: Bearer <JWT>                                 │
│    Body: {"title": "Hello", "content": "..."}                 │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 2. Axum 路由匹配                                               │
│    foundry_engine::router::subsystem_dynamic_routes()         │
│    → handlers::models::create_record()                        │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 3. 认证中间件                                                  │
│    foundry_auth::jwt::verify_token()                          │
│    → 解析 JWT，提取 admin_id 和 role                           │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 4. RBAC 权限检查                                               │
│    foundry_auth::rbac::check_permission()                     │
│    → 验证是否有操作权限                                         │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 5. before_create 钩子                                          │
│    foundry_extension::hooks::trigger_before_create()          │
│    → BlogMutationHook::before_create()                        │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 6. 数据库写入                                                  │
│    foundry_storage::models::RecordStore::create()             │
│    → INSERT INTO records (system_slug, model_slug, data, ...) │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 7. after_create 钩子                                           │
│    foundry_extension::hooks::trigger_after_create()           │
│    → BlogMutationHook::after_create() (发送通知)              │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 8. 审计日志记录                                                │
│    foundry_engine::middleware::audit::log_action()            │
│    → INSERT INTO audit_logs (...)                             │
└───────────────────────────────────────────────────────────────┘
                          ↓
┌───────────────────────────────────────────────────────────────┐
│ 9. 返回响应                                                    │
│    ApiResponse::success(record)                               │
│    → {"success": true, "data": {...}}                         │
└───────────────────────────────────────────────────────────────┘
```

---

### 4.3 子系统自定义路由流程

**场景：** `GET /api/v1/s/blog/custom/stats` 获取博客统计

```rust
// BlogSubsystem 实现
impl SubsystemModule for BlogSubsystem {
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        router.route("/stats", get(handlers::get_blog_stats))
    }
}

// 请求流程：
// 1. 匹配路由 /api/v1/s/blog/custom/stats
// 2. 执行 handlers::get_blog_stats (自定义逻辑)
// 3. 返回自定义响应
```

---

## 5. 学习路径建议

### 5.1 初级阶段（1-2 周）

**目标：** 理解核心概念，能够运行并修改示例项目

**步骤：**

1. **环境搭建**
   ```bash
   # 克隆项目
   git clone https://github.com/foundkit/foundry.git
   cd foundry
   
   # 启动数据库
   cd examples/blog_platform
   docker compose -f dev/docker-compose.yml up -d
   
   # 运行示例
   cargo run
   ```

2. **阅读顺序**
   - [ ] `README.md` - 整体概览
   - [ ] `examples/blog_platform/src/main.rs` - 应用入口
   - [ ] `crates/foundry/src/app.rs` - 理解构建器
   - [ ] `crates/foundry_core/src/subsystem.rs` - 理解子系统接口
   - [ ] `examples/blog_platform/src/systems/blog/mod.rs` - 实战子系统

3. **动手实践**
   - 修改 BlogSubsystem 的名称和描述
   - 添加一个新的配置项
   - 访问管理后台 `http://localhost:3000/admin`

---

### 5.2 中级阶段（2-4 周）

**目标：** 深入理解存储层、认证、Auto-CRUD 机制

**步骤：**

1. **存储层深入**
   - [ ] 阅读 `foundry_storage/src/models.rs`
   - [ ] 理解 `RecordStore` 如何实现通用 CRUD
   - [ ] 查看数据库表结构（`migrations/init.sql`）
   - [ ] 实践：创建一个新的动态模型

2. **认证与授权**
   - [ ] 阅读 `foundry_auth/src/jwt.rs`
   - [ ] 理解 JWT Claims 结构
   - [ ] 实践：添加一个需要特定权限的端点

3. **路由与处理器**
   - [ ] 阅读 `foundry_engine/src/router.rs`
   - [ ] 理解动态路由注册机制
   - [ ] 阅读 `handlers/models.rs` 的 CRUD 实现
   - [ ] 实践：添加一个自定义子系统路由

4. **前端管理后台**
   - [ ] 阅读 `apps/admin/src/App.tsx`
   - [ ] 理解路由配置（`utils/router.ts`）
   - [ ] 实践：修改一个页面的 UI

---

### 5.3 高级阶段（4-8 周）

**目标：** 完全掌控架构，能够扩展核心功能

**步骤：**

1. **生命周期钩子系统**
   - [ ] 阅读 `foundry_extension/src/hooks.rs`
   - [ ] 理解钩子触发时机
   - [ ] 查看 `examples/blog_platform/src/hooks.rs` 示例
   - [ ] 实践：实现一个复杂的钩子（如数据同步）

2. **中间件与拦截器**
   - [ ] 阅读 `foundry_engine/src/middleware/audit.rs`
   - [ ] 理解 Tower 中间件机制
   - [ ] 实践：添加一个自定义中间件（如限流）

3. **数据库迁移与优化**
   - [ ] 理解 SQLx 迁移机制
   - [ ] 实践：添加一个新的原生 SQL 表
   - [ ] 学习索引优化（GIN 索引用于 JSON 查询）

4. **性能优化**
   - [ ] 理解 Redis 缓存策略
   - [ ] 学习连接池配置优化
   - [ ] 实践：为热点数据添加缓存

5. **自定义子系统开发**
   - [ ] 实践：从零创建一个完整子系统（如电商订单系统）
   - [ ] 包含：自定义路由、DTO 验证、业务逻辑、Admin 页面

---

### 5.4 专家阶段（持续学习）

**目标：** 贡献核心代码，参与架构演进

**步骤：**

1. **源码深度分析**
   - 分析每个 Crate 的设计模式
   - 理解跨 Crate 依赖管理
   - 学习 Workspace 最佳实践

2. **性能分析与调优**
   - 使用 `cargo flamegraph` 分析性能瓶颈
   - 优化数据库查询
   - 分析内存使用

3. **扩展核心功能**
   - 添加新的存储后端（如 MySQL）
   - 实现 WebSocket 支持
   - 添加 GraphQL API

4. **参与社区**
   - 提交 PR 修复 Bug
   - 编写文档和教程
   - 回答社区问题

---

## 6. 实践项目

### 6.1 入门项目：待办事项系统

**功能需求：**
- 子系统：TodoSubsystem
- 动态模型：todos (title, completed, priority)
- 自定义路由：`/stats` 返回统计信息
- 钩子：todo 完成后记录日志

**学习要点：**
- Subsystem 注册
- 动态模型 CRUD
- 自定义路由实现
- 钩子基础使用

---

### 6.2 进阶项目：内容管理系统（CMS）

**功能需求：**
- 子系统：ContentSubsystem
- 原生 SQL 表：articles, categories, tags
- 自定义 Admin 页面：富文本编辑器
- RBAC：作者只能编辑自己的文章
- 钩子：文章发布后清除缓存

**学习要点：**
- 混合使用动态模型和原生表
- 复杂权限控制
- 前后端集成
- 缓存失效策略

---

### 6.3 高级项目：电商订单系统

**功能需求：**
- 子系统：OrderSubsystem, ProductSubsystem
- 事务处理：订单创建 + 库存扣减
- 状态机：订单状态流转
- 定时任务：自动取消超时订单
- WebSocket：实时订单状态推送

**学习要点：**
- 跨子系统协作
- 数据一致性保证
- 异步任务处理
- 实时通信

---

## 7. 常见问题与陷阱

### 7.1 常见错误

#### 错误 1：忘记注册子系统

```rust
// ❌ 错误：定义了 BlogSubsystem 但忘记注册
let app = FoundryApp::builder()
    .config(config)
    // .register_subsystem(BlogSubsystem) // 缺失！
    .build()
    .await?;

// ✅ 正确：必须显式注册
let app = FoundryApp::builder()
    .config(config)
    .register_subsystem(BlogSubsystem)
    .build()
    .await?;
```

---

#### 错误 2：动态模型 slug 不匹配

```rust
// ❌ 错误：system_slug 和 model_slug 不一致
RecordStore::create(&db, "blogs", "article", data).await?;
// API 端点：/api/v1/s/blogs/article

// ✅ 正确：确保一致性
RecordStore::create(&db, "blog", "articles", data).await?;
// API 端点：/api/v1/s/blog/articles
```

---

#### 错误 3：JWT Secret 未配置

```bash
# ❌ 错误：未设置 JWT_SECRET 环境变量
# 导致 Token 签名失败

# ✅ 正确：在 .env 中配置
JWT_SECRET=your-super-secret-key-min-32-chars
```

---

### 7.2 性能陷阱

#### 陷阱 1：N+1 查询问题

```rust
// ❌ 不好：循环查询
for article in articles {
    let author = AdminStore::get_by_id(&db, article.author_id).await?;
    // ...
}

// ✅ 更好：批量查询或 JOIN
let authors = AdminStore::get_by_ids(&db, author_ids).await?;
```

---

#### 陷阱 2：未使用 Redis 缓存

```rust
// ❌ 不好：每次都查数据库
let config = ConfigStore::get(&db, "blog", "site_name").await?;

// ✅ 更好：先查缓存
let cache_key = format!("config:blog:site_name");
if let Some(cached) = redis.get(&cache_key).await? {
    return Ok(cached);
}
let config = ConfigStore::get(&db, "blog", "site_name").await?;
redis.set(&cache_key, &config, 3600).await?;
```

---

### 7.3 安全注意事项

1. **密码强度验证**
   - 使用 `validator` crate 强制最小长度
   - 考虑添加密码复杂度检查

2. **CSRF 防护**
   - 对于 SPA，JWT + CORS 已足够
   - 对于传统表单，需要 CSRF Token

3. **SQL 注入防护**
   - 始终使用 SQLx 参数化查询
   - 避免字符串拼接 SQL

4. **XSS 防护**
   - 前端使用 React（自动转义）
   - 后端返回 JSON（避免 HTML 注入）

---

## 8. 学习资源推荐

### 8.1 Rust 基础

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### 8.2 Web 框架

- [Axum Documentation](https://docs.rs/axum/)
- [Tower Middleware](https://docs.rs/tower/)
- [SQLx Guide](https://github.com/launchbadge/sqlx)

### 8.3 数据库

- [PostgreSQL JSON 功能](https://www.postgresql.org/docs/current/datatype-json.html)
- [Redis 命令参考](https://redis.io/commands/)

### 8.4 前端

- [React 官方文档](https://react.dev/)
- [TailwindCSS](https://tailwindcss.com/)

---

## 9. 下一步行动清单

### Week 1-2
- [ ] 搭建本地开发环境
- [ ] 运行 blog_platform 示例
- [ ] 阅读 5 个核心 Crate 的 lib.rs
- [ ] 修改示例项目的配置

### Week 3-4
- [ ] 完成"待办事项系统"实践项目
- [ ] 深入理解 Auto-CRUD 机制
- [ ] 学习 JWT 认证流程

### Week 5-8
- [ ] 完成"内容管理系统"实践项目
- [ ] 实现自定义中间件
- [ ] 优化数据库查询性能

### Week 9-12
- [ ] 完成"电商订单系统"实践项目
- [ ] 为核心 Crate 编写单元测试
- [ ] 尝试贡献一个 PR

---

## 10. 获取帮助

- **GitHub Issues**: [https://github.com/foundkit/foundry/issues](https://github.com/foundkit/foundry/issues)
- **文档站**: [https://foundkit.github.io/foundry/](https://foundkit.github.io/foundry/)
- **讨论区**: [GitHub Discussions](https://github.com/foundkit/foundry/discussions)

---

## 附录 A：关键命令速查表

```bash
# 项目创建
foundry new my-app
cd my-app

# 数据库管理
docker compose -f dev/docker-compose.yml up -d      # 启动数据库
bash dev/init-db.sh                                  # 初始化数据库
docker compose -f dev/docker-compose.yml down -v    # 清理重置

# 子系统管理
foundry system new <slug> --name "<Name>"           # 创建子系统
foundry system list                                  # 列出子系统

# 开发运行
cargo run                                            # 启动服务器
cargo test                                           # 运行测试
cargo build --release                                # 构建发布版本

# 前端开发
cd apps/admin
npm install
npm run dev                                          # 启动开发服务器
npm run build                                        # 构建生产版本
```

---

## 附录 B：环境变量配置

```bash
# .env 示例
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/foundry_dev
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-super-secret-key-at-least-32-characters-long
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
AUTO_MIGRATE=true
RUST_LOG=info,foundry=debug
```

---

**祝学习顺利！掌控 Foundry 框架后，你将拥有快速构建模块化后端系统的能力。** 🚀

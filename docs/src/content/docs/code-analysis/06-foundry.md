---
title: "06. Facade Layer & Assembly (foundry)"
description: "Stage 6: In-depth analysis of FoundryApp, FoundryBuilder pattern, and bootstrap lifecycle."
---

> **Codebase Navigation**：⬅️ Previous: [05. Request Engine & Auto-CRUD (foundry_engine)](./05-engine/) · [📋 Overview & Guide](./) · ➡️ Next: [07. Blog Platform Full-Stack Example (blog_platform)](./07-blog-platform/)

# Foundry 门面层代码深度分析

> 第六阶段：理解门面层（应用组装与启动）
>
> Crate: `foundry`
>
> 阅读时间：30-45 分钟

---

## 概述

`foundry` 是框架的**门面 crate**，提供：
- **统一的 API 入口**（FoundryApp、FoundryBuilder）
- **构建器模式**（流式配置）
- **应用组装逻辑**（连接所有底层组件）
- **Prelude 便捷导入**（一站式导入所有常用类型）

**这是用户直接使用的唯一 crate，隐藏了底层复杂性。**

**依赖关系：**
```
foundry_core, foundry_storage, foundry_auth, foundry_extension, foundry_engine
                           ↓
                       foundry (门面)
                           ↓
                     用户应用代码
```

---

## 1. 架构设计

### 1.1 门面模式（Facade Pattern）

```
┌──────────────────────────────────────────┐
│            foundry (Facade)              │
│  ┌────────────────────────────────────┐ │
│  │  FoundryApp / FoundryBuilder       │ │
│  │  prelude (统一导出)                │ │
│  └────────────────────────────────────┘ │
└──────────────────────────────────────────┘
              ↓ 封装
┌──────────────────────────────────────────┐
│      7 个底层 Crates（复杂实现）          │
│  foundry_core, foundry_storage,          │
│  foundry_auth, foundry_extension,        │
│  foundry_engine...                       │
└──────────────────────────────────────────┘
```

**优势：**
- 用户只需依赖 `foundry` 一个 crate
- 简化的 API（构建器模式）
- 底层变更不影响用户代码

### 1.2 特性标志（Feature Flags）

```toml
[features]
default = ["server", "storage", "auth", "extension"]
server = ["foundry_engine", "axum", "tower", "tokio"]
storage = ["foundry_storage", "sqlx"]
auth = ["foundry_auth"]
extension = ["foundry_extension"]
```

**灵活性：**
```toml
# 完整功能（默认）
[dependencies]
foundry = "0.1"

# 仅核心类型（不启动服务器）
[dependencies]
foundry = { version = "0.1", default-features = false }
```

---

## 2. FoundryConfig - 配置结构

### 2.1 配置字段

```rust
#[derive(Debug, Clone)]
pub struct FoundryConfig {
    pub host: String,                          // 监听地址（默认 0.0.0.0）
    pub port: u16,                             // 端口（默认 8080）
    pub database_url: String,                  // PostgreSQL 连接字符串
    pub redis_url: Option<String>,             // Redis 连接字符串（可选）
    pub jwt_secret: String,                    // JWT 签名密钥
    pub jwt_expiry_hours: i64,                 // Token 有效期（默认 7 天）
    pub auto_migrate: bool,                    // 自动执行迁移（默认 true）
    pub db_pool_size: u32,                     // 连接池大小（默认 20）
    pub external_systems_dirs: Vec<PathBuf>,   // 外部子系统目录
    pub admin_static_dirs: Vec<PathBuf>,       // Admin UI 静态资源目录
}
```

### 2.2 默认值（从环境变量）

```rust
impl Default for FoundryConfig {
    fn default() -> Self {
        Self {
            host: env::var("HOST").unwrap_or("0.0.0.0".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL").unwrap_or(
                "postgres://postgres:password@localhost:5432/foundry".into()
            ),
            redis_url: env::var("REDIS_URL").ok(),
            jwt_secret: env::var("JWT_SECRET").unwrap_or(
                "foundry_super_secret_jwt_key_2026_change_in_production".into()
            ),
            jwt_expiry_hours: 24 * 7,  // 7 天
            auto_migrate: env::var("AUTO_MIGRATE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            db_pool_size: 20,
            external_systems_dirs: Vec::new(),
            admin_static_dirs: Vec::new(),
        }
    }
}
```

### 2.3 从环境加载

```rust
impl FoundryConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();  // ← 加载 .env 文件
        Self::default()
    }
}
```

**使用示例：**

```bash
# .env 文件
DATABASE_URL=postgresql://user:pass@localhost/mydb
REDIS_URL=redis://localhost:6379
JWT_SECRET=my-super-secret-key-32-bytes-long
JWT_EXPIRY_HOURS=48
AUTO_MIGRATE=true
HOST=127.0.0.1
PORT=3000
```

```rust
let config = FoundryConfig::from_env();
```

---

## 3. FoundryBuilder - 构建器模式

### 3.1 结构定义

```rust
pub struct FoundryBuilder {
    config: FoundryConfig,
    subsystems: Vec<Box<dyn SubsystemModule>>,
    hooks: HookPipeline,
    init_tracing: bool,
}
```

### 3.2 构建器方法

#### A. 配置方法（链式调用）

```rust
impl FoundryBuilder {
    pub fn new() -> Self { /* ... */ }

    // 直接设置配置对象
    pub fn config(mut self, config: FoundryConfig) -> Self {
        self.config = config;
        self
    }

    // 单独设置配置项
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn database_url(mut self, url: impl Into<String>) -> Self {
        self.config.database_url = url.into();
        self
    }

    pub fn redis_url(mut self, url: Option<String>) -> Self {
        self.config.redis_url = url;
        self
    }

    pub fn jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.jwt_secret = secret.into();
        self
    }

    pub fn jwt_expiry_hours(mut self, hours: i64) -> Self {
        self.config.jwt_expiry_hours = hours;
        self
    }

    pub fn auto_migrate(mut self, auto: bool) -> Self {
        self.config.auto_migrate = auto;
        self
    }

    pub fn db_pool_size(mut self, size: u32) -> Self {
        self.config.db_pool_size = size;
        self
    }

    pub fn init_tracing(mut self, init: bool) -> Self {
        self.init_tracing = init;
        self
    }
}
```

#### B. 注册方法

```rust
impl FoundryBuilder {
    // 注册子系统
    pub fn register_subsystem<S: SubsystemModule>(mut self, subsystem: S) -> Self {
        self.subsystems.push(Box::new(subsystem));
        self
    }

    pub fn register_boxed_subsystem(mut self, subsystem: Box<dyn SubsystemModule>) -> Self {
        self.subsystems.push(subsystem);
        self
    }

    // 注册钩子
    pub fn register_hook<H: MutationHook>(mut self, hook: H) -> Self {
        self.hooks.register(hook);
        self
    }

    // 外部子系统目录
    pub fn with_external_systems_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.external_systems_dirs.push(path.into());
        self
    }

    // Admin UI 静态资源目录
    pub fn with_admin_spa_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.admin_static_dirs.push(path.into());
        self
    }
}
```

### 3.3 构建方法（build）

```rust
impl FoundryBuilder {
    pub async fn build(self) -> AppResult<FoundryApp> {
        // 1. 初始化日志
        if self.init_tracing {
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env()
                    .unwrap_or("info,foundry=debug,foundry_engine=debug".into()))
                .with(fmt::layer())
                .try_init();
        }

        info!("Initializing Foundry Platform Application...");

        // 2. 初始化 PostgreSQL 连接池
        info!("Connecting to PostgreSQL database...");
        let db_pool = init_db_pool(
            &self.config.database_url,
            self.config.db_pool_size
        ).await?;

        // 3. 运行数据库迁移
        if self.config.auto_migrate {
            run_migrations(&db_pool).await?;
        }

        // 4. 初始化 Redis（可选）
        let redis_pool = if let Some(ref rurl) = self.config.redis_url {
            match init_redis(rurl).await {
                Ok(pool) => Some(pool),
                Err(e) => {
                    tracing::warn!("Redis failed: {}. Continuing without Redis.", e);
                    None
                }
            }
        } else {
            None
        };

        // 5. 创建 JWT 服务
        let jwt_service = JwtService::new(
            self.config.jwt_secret.clone(),
            self.config.jwt_expiry_hours
        );

        // 6. 收集子系统（代码 + 外部）
        let mut all_subsystems = self.subsystems;
        let external_subsystems = load_external_subsystems(
            &self.config.external_systems_dirs
        );
        all_subsystems.extend(external_subsystems);

        info!("Loaded {} active sub-systems", all_subsystems.len());

        // 7. 创建应用状态
        let state = AppState::new(
            db_pool,
            redis_pool,
            jwt_service,
            self.hooks,
            all_subsystems
        );

        // 8. 构建路由
        let router = build_router(state.clone());

        Ok(FoundryApp {
            config: self.config,
            state,
            router,
        })
    }
}
```

**构建流程图：**

```
FoundryBuilder::new()
    ↓
配置（host, port, database_url...）
    ↓
注册子系统（register_subsystem）
    ↓
注册钩子（register_hook）
    ↓
build()
    ↓
┌──────────────────────────────────┐
│ 1. 初始化日志（tracing）         │
├──────────────────────────────────┤
│ 2. 连接 PostgreSQL               │
├──────────────────────────────────┤
│ 3. 执行数据库迁移                │
├──────────────────────────────────┤
│ 4. 连接 Redis（可选）            │
├──────────────────────────────────┤
│ 5. 创建 JwtService               │
├──────────────────────────────────┤
│ 6. 加载外部子系统                │
├──────────────────────────────────┤
│ 7. 创建 AppState                 │
├──────────────────────────────────┤
│ 8. 构建路由树（build_router）   │
└──────────────────────────────────┘
    ↓
FoundryApp（可运行）
```

---

## 4. FoundryApp - 应用实例

### 4.1 结构定义

```rust
pub struct FoundryApp {
    pub config: FoundryConfig,
    pub state: AppState,
    pub router: Router,
}
```

### 4.2 便捷方法

```rust
impl FoundryApp {
    // 创建构建器
    pub fn builder() -> FoundryBuilder {
        FoundryBuilder::new()
    }

    // 获取路由副本
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    // 消耗 self，返回路由
    pub fn into_router(self) -> Router {
        self.router
    }

    // 获取数据库连接池
    pub fn db_pool(&self) -> &DbPool {
        &self.state.db
    }

    // 获取 Redis 连接池
    pub fn redis_pool(&self) -> Option<&RedisPool> {
        self.state.redis.as_ref()
    }
}
```

### 4.3 运行方法

#### A. run() - 使用配置的地址

```rust
impl FoundryApp {
    pub async fn run(self) -> anyhow::Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid listen address: {}", e)))?;
        
        self.serve(addr).await
    }
}
```

#### B. serve() - 使用自定义地址

```rust
impl FoundryApp {
    pub async fn serve(self, addr: SocketAddr) -> anyhow::Result<()> {
        info!("🚀 Foundry Application listening on http://{}", addr);
        info!("📚 REST APIs available at http://{}/api/v1", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}
```

---

## 5. prelude - 统一导入

### 5.1 设计理念

**问题：** 用户需要从多个 crate 导入类型，繁琐且容易出错。

```rust
// ❌ 不便：需要知道每个类型来自哪个 crate
use foundry_core::{AppError, ApiResponse};
use foundry_storage::{RecordStore, DbPool};
use foundry_auth::{JwtService, AdminClaims};
use foundry_extension::MutationHook;
```

**解决方案：** 统一的 `prelude` 模块

```rust
// ✅ 便捷：一次导入所有常用类型
use foundry::prelude::*;
```

### 5.2 导出内容

```rust
pub use crate::app::{FoundryApp, FoundryBuilder, FoundryConfig};

pub use async_trait::async_trait;

pub use foundry_auth::{
    AdminClaims, JwtService, check_system_access, hash_password, verify_password,
};

pub use foundry_core::{
    ApiResponse, AppError, AppResult, CustomAdminPageSpec, FieldType, PageMeta,
    PaginatedData, SubsystemModule, SystemContext, SystemStatus, is_valid_slug,
};

pub use foundry_engine::{
    AppState, ExternalSubsystemManifest, ExternalSubsystemModule,
};

pub use foundry_extension::{HookPipeline, MutationHook};

pub use foundry_storage::{
    AdminStore, AuditLogInsert, AuditLogQuery, AuditStore, ConfigStore, DbPool,
    ModelStore, RecordQuery, RecordStore, RedisPool, SystemQuery, SystemStore,
    init_db_pool, init_redis, run_migrations,
};
```

**分类：**

| 类别 | 导出内容 |
|------|---------|
| **应用入口** | FoundryApp, FoundryBuilder, FoundryConfig |
| **核心类型** | AppError, ApiResponse, SystemContext, SubsystemModule |
| **认证** | JwtService, AdminClaims, hash_password, verify_password |
| **存储** | RecordStore, ModelStore, SystemStore, AdminStore, ConfigStore |
| **扩展** | MutationHook, HookPipeline |
| **工具** | async_trait, is_valid_slug |

---

## 6. 完整使用示例

### 6.1 最简示例

```rust
use foundry::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = FoundryApp::builder()
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

**效果：**
- 从环境变量读取配置
- 连接数据库
- 自动执行迁移
- 启动服务器（默认 `0.0.0.0:8080`）

### 6.2 完整示例

```rust
use foundry::prelude::*;

mod systems;
mod hooks;

use systems::{BlogSubsystem, ShopSubsystem};
use hooks::{BlogHook, NotificationHook};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = FoundryConfig::from_env();

    let app = FoundryApp::builder()
        // 自定义配置
        .config(config)
        .host("127.0.0.1")
        .port(3000)
        .jwt_expiry_hours(48)
        .db_pool_size(30)
        
        // 注册子系统
        .register_subsystem(BlogSubsystem)
        .register_subsystem(ShopSubsystem)
        
        // 注册钩子
        .register_hook(BlogHook)
        .register_hook(NotificationHook)
        
        // 外部子系统
        .with_external_systems_dir("plugins/")
        
        // Admin UI 自定义目录
        .with_admin_spa_dir("frontend/dist")
        
        // 构建
        .build()
        .await?;

    // 运行
    app.run().await?;
    Ok(())
}
```

### 6.3 集成测试示例

```rust
use foundry::prelude::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for oneshot()

#[tokio::test]
async fn test_health_check() {
    let app = FoundryApp::builder()
        .database_url("postgresql://...")
        .build()
        .await
        .unwrap();

    let router = app.into_router();

    let response = router
        .oneshot(Request::builder().uri("/api/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## 7. 设计模式总结

### 7.1 门面模式（Facade Pattern）

```
foundry crate
    ↓ 简化接口
7 个复杂的底层 crates
```

### 7.2 构建器模式（Builder Pattern）

```rust
FoundryApp::builder()
    .host("0.0.0.0")
    .port(8080)
    .register_subsystem(MySubsystem)
    .build()
    .await?
```

**优势：**
- 可选参数（不需要长参数列表）
- 链式调用（可读性强）
- 类型安全（编译时检查）

### 7.3 默认值模式（Default Pattern）

```rust
impl Default for FoundryConfig {
    fn default() -> Self {
        // 从环境变量读取，或使用合理默认值
    }
}
```

### 7.4 Prelude 模式（Prelude Pattern）

```rust
pub mod prelude;  // 统一导入入口
```

**标准库也使用此模式：**
```rust
use std::prelude::*;  // 自动导入
```

---

## 8. 错误处理

### 8.1 构建时错误

```rust
let app = FoundryApp::builder()
    .database_url("invalid_url")
    .build()
    .await?;  // ← 返回 AppResult<FoundryApp>

// 可能的错误：
// - Database connection failed
// - Migration failed
// - Redis connection failed (会警告但不中断)
// - Invalid configuration
```

### 8.2 运行时错误

```rust
app.run().await?;  // ← 返回 anyhow::Result<()>

// 可能的错误：
// - Invalid listen address
// - Port already in use
// - Permission denied
```

---

## 9. 最佳实践

### 9.1 环境变量管理

```bash
# .env.example（提交到 Git）
DATABASE_URL=postgresql://user:pass@localhost/foundry
REDIS_URL=redis://localhost:6379
JWT_SECRET=change-me-in-production
AUTO_MIGRATE=true

# .env（不提交到 Git）
DATABASE_URL=postgresql://prod_user:secret@prod-db/foundry
JWT_SECRET=actual-production-secret-key
```

### 9.2 配置分离

```rust
// 开发环境
#[cfg(debug_assertions)]
let config = FoundryConfig::from_env();

// 生产环境
#[cfg(not(debug_assertions))]
let config = FoundryConfig {
    auto_migrate: false,  // 生产环境手动迁移
    // ...
};
```

### 9.3 优雅关闭

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = FoundryApp::builder().build().await?;
    
    // 监听 Ctrl+C
    tokio::select! {
        result = app.run() => {
            result?;
        }
        _ = signal::ctrl_c() => {
            info!("Shutting down gracefully...");
        }
    }
    
    Ok(())
}
```

---

## 10. 与其他 Crates 的关系

```
用户代码
    ↓ 使用
foundry (门面)
    ↓ 依赖
┌──────────────────────────────────┐
│ foundry_core                     │ ← 基础类型
├──────────────────────────────────┤
│ foundry_storage                  │ ← 数据访问
├──────────────────────────────────┤
│ foundry_auth                     │ ← 认证授权
├──────────────────────────────────┤
│ foundry_extension                │ ← 扩展机制
├──────────────────────────────────┤
│ foundry_engine                   │ ← 请求处理
└──────────────────────────────────┘
```

---

## 11. 快速参考

### 最小化启动

```rust
use foundry::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    FoundryApp::builder().build().await?.run().await
}
```

### 完整配置

```rust
FoundryApp::builder()
    .host("0.0.0.0")
    .port(8080)
    .database_url("postgresql://...")
    .redis_url(Some("redis://...".into()))
    .jwt_secret("secret")
    .jwt_expiry_hours(48)
    .auto_migrate(true)
    .db_pool_size(20)
    .register_subsystem(MySubsystem)
    .register_hook(MyHook)
    .build()
    .await?
    .run()
    .await
```

---

**✅ 第六阶段完成！** 你现在应该理解了：
- FoundryConfig 的所有配置选项和默认值
- FoundryBuilder 的构建器模式（链式调用）
- FoundryApp 的构建和运行流程
- prelude 的设计和导出内容
- 完整的应用启动过程（8 个步骤）
- 设计模式（门面、构建器、默认值、Prelude）

**接下来的两个阶段：**
- 第七阶段：示例应用（blog_platform）- 看实际使用
- 第八阶段：前端管理后台（apps/admin）- 看 UI 实现

准备继续吗？
---

### 📚 Stage Navigation
- ⬅️ **Previous Stage**: [05. Request Engine & Auto-CRUD (foundry_engine)](./05-engine/)
- 📋 **Guide Overview**: [Return to Overview](./)
- ➡️ **Next Stage**: [07. Blog Platform Full-Stack Example (blog_platform)](./07-blog-platform/)

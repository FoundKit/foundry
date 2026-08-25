---
title: 快速入门
description: 使用 Foundry 框架创建、构建和运行独立应用程序的快速入门指南。
---

# Foundry 快速入门

Foundry 是一个现代、模块化、彻底解耦的 Rust 后端平台与开发框架。通过将平台底层能力（运行时、路由、存储、鉴权、Admin 外壳）与业务应用彻底分离，使开发者能够使用 Cargo 依赖快速构建具备完整后台管理能力的现代化业务服务。

---

## ⚡ 5 分钟上手

### 1. 安装 Foundry CLI

使用 Cargo 全局安装 `foundry-cli` 工具：

```bash
cargo install foundry-cli
```

验证安装是否成功：

```bash
foundry --help
```

---

### 2. 创建独立应用项目

使用 `foundry new` 创建全新的独立应用项目：

```bash
foundry new my-app
cd my-app
```

生成的项目为一个完全独立的 Rust 应用程序：

```text
my-app/
├── Cargo.toml                # 依赖 foundry = "0.1"
├── src/
│   ├── main.rs               # 应用启动入口 (FoundryApp::builder)
│   └── systems/
│       ├── mod.rs
│       └── sample/           # 默认初始业务子系统
│           ├── controllers/  # 自定义 Axum 路由 (/api/v1/s/sample/ext/*)
│           ├── logic/        # 业务领域服务
│           ├── dto/          # 请求 DTO 与字段校验
│           ├── custom_pages/ # 自定义 Admin 运营看板 (HTML/React)
│           └── mod.rs
├── migrations/               # 业务专属数据库迁移脚本
├── .env                      # 本地环境配置
└── README.md
```

---

### 3. 启动数据库与缓存服务

使用 Docker 快速启动本地 PostgreSQL 与 Redis：

```bash
docker run -d --name foundry-postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry -p 5432:5432 postgres:18-alpine
docker run -d --name foundry-redis -p 6379:6379 redis:8-alpine
```

---

### 4. 运行应用程序

```bash
cargo run
```

Foundry 将自动连接数据库，初始化基础元数据表，自动挂载已注册的业务子系统路由，并在 `http://127.0.0.1:8080` 启动监听。

---

## 🏗️ 应用程序如何使用 Foundry

在用户应用的 `Cargo.toml` 中：

```toml
[dependencies]
foundry = "0.1"
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
```

在 `src/main.rs` 中：

```rust
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 读取环境配置
    let config = FoundryConfig::from_env();

    // 2. 构造 Foundry 运行时并注册子系统
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(SampleSubsystem)
        .build()
        .await?;

    // 3. 启动服务
    app.run().await?;
    Ok(())
}
```

---

## 🧩 扩展业务子系统

在项目中快速脚手架一个新的业务子系统：

```bash
foundry system new blog --name "博客与内容发布"
```

在 `src/main.rs` 中注册：

```rust
use systems::BlogSubsystem;

let app = FoundryApp::builder()
    .register_subsystem(BlogSubsystem)
    // ...
    .build()
    .await?;
```

新子系统的接口即刻在 `/api/v1/s/blog/ext/*` 生效。

---

## 📦 下一步

- 阅读 [架构设计蓝图](../architecture/blueprint/) 理解 Framework 与 Application 的物理隔离设计。
- 查阅 [子系统与扩展开发](../guides/extensions/) 学习如何编写自定义控制器、领域服务和 Admin 扩展大屏。
- 查阅 [路线图与版本策略](../roadmap/) 了解 SemVer 版本演进与升级方法。

---
title: 快速入门
description: 基于 Git 依赖创建、构建并启动独立 Foundry 应用的 5 分钟上手指南。
---

# Foundry 快速入门

Foundry 是一个现代、模块化、完全解耦的 Rust 后端平台与开发框架。在当前测试与验证阶段（正式发布至 crates.io 前），开发者可以通过 **Git 依赖** 的方式在自己的独立仓库中引入并使用 Foundry。

---

## ⚡ 5 分钟快速上手

### 1. 安装 Foundry CLI

通过 Cargo 直接从 GitHub 仓库安装 `foundry` CLI 命令行工具：

```bash
# 从 GitHub 仓库安装
cargo install --git https://github.com/foundkit/foundry foundry_cli
```

验证 CLI 工具是否安装成功：

```bash
foundry --help
```

> **本地开发提示**：如果你已经将 Foundry 源码 clone 到本地，也可以直接通过本地路径安装：
> ```bash
> cargo install --path crates/foundry_cli
> ```

---

### 2. 创建独立应用工程

使用 `foundry new` 创建全新的独立应用工程：

```bash
foundry new my-app
cd my-app
```

生成的工程为一个完全独立的 Rust 应用程序，且已预先配置好 Git 依赖：

```text
my-app/
├── Cargo.toml                # 预先配置了 foundry = { git = "...", branch = "main" }
├── src/
│   ├── main.rs               # 应用启动入口 (FoundryApp::builder)
│   └── systems/
│       ├── mod.rs
│       └── sample/           # 示例业务子系统
│           ├── controllers/  # 自定义 Axum 路由 (/api/v1/s/sample/ext/*)
│           ├── logic/        # 领域业务逻辑
│           ├── dto/          # 请求参数与 validator 校验
│           ├── custom_pages/ # 自定义 Admin 运营看板 (HTML/React)
│           └── mod.rs
├── migrations/               # 业务专属数据库迁移脚本
├── .env                      # 本地环境变量配置
└── README.md
```

#### 查看生成的 `Cargo.toml`：

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8", features = ["macros"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
```

---

### 3. 启动数据库与缓存服务

使用 Docker 快速启动本地 PostgreSQL 与 Redis 实例：

```bash
docker run -d --name foundry-postgres \
  -e POSTGRES_PASSWORD=postgrespassword \
  -e POSTGRES_DB=foundry \
  -p 5432:5432 \
  postgres:17-alpine

docker run -d --name foundry-redis \
  -p 6379:6379 \
  redis:7-alpine
```

检查项目根目录下的 `.env` 配置是否与本地环境匹配：

```bash
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

---

### 4. 运行应用程序

在项目目录下启动后端服务：

```bash
cargo run
```

服务启动时，Foundry 将自动完成以下初始化：
1. 连接 PostgreSQL 数据库与 Redis 缓存。
2. 自动执行基础元数据表的迁移（`AUTO_MIGRATE=true` 时）。
3. 挂载已注册的所有业务子系统路由。
4. 监听并在 `http://0.0.0.0:8080`（可通过 `http://localhost:8080` 访问）提供服务。

---

### 5. 打开后台管理控制台

在浏览器中直接访问：

```text
http://localhost:8080/admin
```

#### 默认管理员账号与密码：
* **用户名**: `admin`
* **初始密码**: `admin123456`
* **角色**: `super_admin`（超级管理员）

#### 使用 CLI 创建新管理员账户：
开发过程中，你可以随时通过 CLI 工具创建新管理员：

```bash
foundry admin create --username developer --password devsecret --role super_admin
```

---

### 6. 验证接口调用

#### 健康检查接口：
```bash
curl http://localhost:8080/api/v1/health
# 返回: OK
```

#### 示例业务子系统接口：
```bash
curl -X POST http://localhost:8080/api/v1/s/sample/ext/greet \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'

# 返回:
# {
#   "code": 0,
#   "message": "success",
#   "data": {
#     "message": "Hello, Alice! Welcome to Foundry Framework."
#   }
# }
```

---

## 🧩 下一步指南

- **[数据库与自定义存储开发](../guides/database/)**：学习如何在自定义业务代码中执行原生 SQL、事务操作、Zero-DDL 动态模型读写与编写迁移。
- **[子系统与自定义功能开发](../guides/extensions/)**：学习三层架构开发模式与 Admin 运营大屏集成。
- **[CLI 命令行工具指南](../guides/cli/)**：全面掌握脚手架与运维命令。

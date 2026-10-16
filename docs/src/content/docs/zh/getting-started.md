---
title: 快速入门
description: 基于 Git 依赖创建、构建并启动独立 Foundry 应用的 5 分钟上手指南，全面支持 PostgreSQL、MySQL、MariaDB 与 MongoDB。
---

> **初学者友好指南**：本篇指南专为首次接触 Foundry 的开发者设计。无需深奥的底层架构知识，只需跟随以下步骤，你将在 **5 分钟内** 完成环境准备、工程创建、首个业务接口编写与本地服务启动！

Foundry 是一个现代、模块化、完全解耦的 Rust 后端平台与开发框架。在当前测试与验证阶段，开发者可以通过 **Git 依赖** 的方式在自己的独立仓库中引入并使用 Foundry。

```mermaid
flowchart LR
    Step1["1. 安装 CLI 工具"] --> Step2["2. 创建独立应用工程"]
    Step2 --> Step3["3. 启动本地数据库"]
    Step3 --> Step4["4. 启动服务 & 访问控制台"]
    Step4 --> Step5["5. 编写首个业务接口"]
```

> [!NOTE]
> **原生企业级多数据库 SPI 驱动支持**：Foundry 通过统一零驱动泄露的 `Database` 门面与 SPI 架构，原生支持四大主流数据库引擎：
> 1. **PostgreSQL (14+)** *(默认引擎)* — 关系型存储引擎，深度支持原生 JSONB 与 GIN 倒排索引。
> 2. **MongoDB (6.0+)** — 原生 BSON 文档引擎，支持原子递增序列计数器与通配符索引（Zero-DDL 无需繁琐建表）。
> 3. **MySQL (8.0+)** — 通用单表关系型存储引擎，具备覆盖索引与延迟关联分页优化。
> 4. **MariaDB (10.5+)** — 专属 MariaDB 方言引擎，具备精细化 InnoDB 索引适配。

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

> [!TIP]
> **本地开发提示**：如果你已经将 Foundry 源码 clone 到本地，也可以直接通过本地路径安装：
> ```bash
> cargo install --path crates/foundry_cli
> ```

---

### 2. 创建独立应用工程

使用 `foundry new` 创建全新的独立应用工程。你可通过 `--db`（或 `-d`）参数选择目标底层数据库引擎：

```bash
# 选项 1：默认引擎 (PostgreSQL)
foundry new my-app

# 选项 2：MongoDB (Zero-DDL 文档引擎，免去 SQL 迁移烦恼)
foundry new my-app --db mongodb

# 选项 3：MySQL (8.0+)
foundry new my-app --db mysql

# 选项 4：MariaDB (10.5+)
foundry new my-app --db mariadb

cd my-app
```

CLI 脚手架将自动生成一个完全独立的 Rust 项目，且已预先匹配好对应数据库的 Cargo 特性开关与环境变量：

```text
my-app/
├── Cargo.toml                # 预先配置好对应数据库 feature 与 foundry 依赖
├── src/
│   ├── main.rs               # 应用启动入口 (FoundryApp::builder)
│   └── systems/
│       ├── mod.rs
│       └── sample/           # 示例业务子系统
│           ├── controllers/  # 自定义 Axum 路由 (/api/v1/s/sample/ext/*)
│           ├── logic/        # 领域业务逻辑
│           ├── dto/          # 请求参数与 validator 校验
│           ├── custom_pages/ # 自定义 Admin 运营看板 (HTML / React iframe)
│           └── mod.rs
├── migrations/               # （可选）按需导出的方言迁移脚本目录 (执行 `foundry ddl` 时生成)
├── .env                      # 本地环境变量配置 (DATABASE_URL, DATABASE_TYPE, AUTO_MIGRATE 开关)
├── .env.example              # 环境变量配置模板
└── README.md
```

#### 查看生成的 `Cargo.toml`：

Foundry 秉承 **“极简内核，开箱即用”** 的哲学：底层 Web 路由、HTTP 中间件、JSON 序列化、追踪日志及错误信封等基础设施全部由官方 `foundry` 门面 crate 内聚封装并重导出（`foundry::prelude::*`）。

新工程**仅需 3 个核心依赖**，无任何臃肿冗余依赖干扰：

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
# 1. 核心框架门面（根据需求开启对应的数据库特性）
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# 2. 异步运行时与模型序列化
tokio = { version = "1.44", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

---

### 3. 启动并初始化数据库与缓存服务

Foundry 应用通过根目录下 `.env` 的 `DATABASE_URL` 与 `DATABASE_TYPE` 连接底层数据库，并使用 Redis 作为租户隔离缓存与分布式锁。

#### 方式一：使用 Docker 启动本地容器 (最快捷)

在本地快速体验时，可直接通过 Docker 启动所需的数据库与 Redis 7 容器：

```bash
# 1. 启动 Redis 7 缓存服务
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# --- 请选择以下四种数据库之一启动 ---

# 选项 A: PostgreSQL 18 (默认)
docker run -d --name my-postgres -p 5432:5432 \
  -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry \
  postgres:18-alpine

# 选项 B: MongoDB 7 (Zero-DDL 原生文档引擎)
docker run -d --name my-mongodb -p 27017:27017 \
  mongo:7

# 选项 C: MySQL 8.4
docker run -d --name my-mysql -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry \
  mysql:8.4

# 选项 D: MariaDB 11
docker run -d --name my-mariadb -p 3307:3306 \
  -e MARIADB_ROOT_PASSWORD=root -e MARIADB_DATABASE=foundry \
  mariadb:11
```

#### 方式二：配置环境变量 (`.env`)

检查项目根目录下的 `.env` 配置，根据实际所使用的数据库调整：

```bash
HOST=0.0.0.0
PORT=8080

# 选项 A: PostgreSQL (默认)
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
DATABASE_TYPE=postgres

# 缓存与安全
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production

# 自动建表与超管初始化 (开发环境保持 true)
AUTO_MIGRATE=true
```

---

### 4. 运行应用程序

在项目根目录下启动后端服务：

```bash
cargo run
```

服务启动时，Foundry 将全自动完成以下初始化流程：
1. 智能连接所配置的底层数据库与 Redis。
2. 自动检查数据库元数据并创建所需表与索引，确保初始超级管理员账号就绪（`AUTO_MIGRATE=true`）。
3. 挂载内置的嵌入式 React 管理后台与所有业务子系统自定义路由。
4. 在 `http://0.0.0.0:8080` 开始对外提供服务。

---

### 5. 打开后台管理控制台

在浏览器中直接访问管理路径：

```text
http://localhost:8080/admin/
```

#### 默认管理员账号与密码：
* **用户名**: `admin`
* **初始密码**: `admin123456`
* **角色**: `super_admin`（超级管理员）

在管理控制台中，你可以可视化查看当前注册的全部业务子系统、配置系统参数、添加管理员或直接管理 Zero-DDL 动态模型。

---

### 6. 验证接口调用

#### 1. 系统健康检查：
```bash
curl http://localhost:8080/health
# 返回: OK
```

#### 2. 超管登录与 JWT 获取：
```bash
curl -X POST http://localhost:8080/api/v1/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123456"}'
```

#### 3. 示例业务子系统接口：
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

## 🛠️ 3 分钟实战：编写你的首个业务接口

为帮助初学者迅速体会 Foundry 的开发手感，我们为现有的 `sample` 子系统增加一个**计算器接口**：

### 步骤 1：在 `src/systems/sample/controllers/mod.rs` 增加处理函数

```rust
use foundry::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    pub a: f64,
    pub b: f64,
    pub op: String, // "add", "sub", "mul", "div"
}

// 编写你的异步控制器 Handler
pub async fn handle_calc(
    Json(payload): Json<CalcRequest>,
) -> AppResult<ApiResponse<f64>> {
    let result = match payload.op.as_str() {
        "add" => payload.a + payload.b,
        "sub" => payload.a - payload.b,
        "mul" => payload.a * payload.b,
        "div" => {
            if payload.b == 0.0 {
                return Err(AppError::BadRequest("除数不能为零".into()));
            }
            payload.a / payload.b
        }
        _ => return Err(AppError::BadRequest("未知运算符".into())),
    };

    Ok(ApiResponse::ok(result))
}
```

### 步骤 2：绑定路由

在同一个文件的 `build_routes()` 中追加一行 `.route("/calc", post(handle_calc))`：

```rust
pub fn build_routes() -> Router {
    Router::new()
        .route("/greet", post(handle_greet))
        .route("/calc", post(handle_calc)) // 新增这行
}
```

### 步骤 3：测试运行

保存后重新执行 `cargo run`，使用 `curl` 即可调用你的新接口：

```bash
curl -X POST http://localhost:8080/api/v1/s/sample/ext/calc \
  -H "Content-Type: application/json" \
  -d '{"a": 100, "b": 25, "op": "div"}'

# 返回：
# {"code":0,"message":"success","data":4.0,"meta":null}
```

恭喜你！你已经成功编写并运行了属于自己的第一个 Foundry API！

---

## ❓ 新手常见疑问与避坑 FAQ

> [!TIP]
> **Q1: 为什么我的代码里找不到 Axum 或 Tokio 的某些类型？**
> **答**：只需一条 `use foundry::prelude::*;`，框架已统一重导出了 `Router`、`Json`、`Query`、`Path`、`Extension`、`ApiResponse`、`AppResult`、`info!` 等 95% 的日常类型，无需在 `Cargo.toml` 中显式引入零碎库。
>
> **Q2: 本地没有安装 Redis，可以运行吗？**
> **答**：可以！在 `.env` 中留空 `REDIS_URL=`，Foundry 会自动安全降级为内存模式，绝不会导致服务崩溃。
>
> **Q3: 为什么第一次 `cargo run` 比较慢？**
> **答**：Rust 在首次构建时需要编译底层高性能网络栈与数据库驱动。后续增量编译通常只需 1~2 秒即可完成热构建。

---

## 🧩 系统进阶教程 (模块化学习路径)

按需查阅我们的全新模块化教程：
* 🏗️ **[01. 基础架构与生命周期](../development/01-architecture-lifecycle/)**：理解底层解耦机制与时序流转
* 🛣️ **[02. 路由系统与子系统路由](../development/02-routing/)**：掌握子系统命名空间与参数绑定
* 🎮 **[03. 控制器、参数与响应规范](../development/03-controllers/)**：熟练编写 Thin Controller
* 🛡️ **[04. 请求验证器与 DTO 设计](../development/04-validation/)**：使用 validator 宏实现强校验
* 💾 **[05. 数据库配置与 Code-First ORM](../development/05-database-orm/)**：类型安全实体定义与链式查询
* ⚡ **[06. Zero-DDL 动态数据模型](../development/06-dynamic-models/)**：免写 SQL 即刻获得 Auto-CRUD
* 🔐 **[07. 认证安全与分级 RBAC](../development/07-auth-security/)**：JWT 与三级角色权限控制
* 🪝 **[08. 生命周期突变钩子](../development/08-mutation-hooks/)**：拦截实体创建、更新与删除
* 🖥️ **[09. 管理控制台与 postMessage 桥接](../development/09-admin-integration/)**：将自定义运营大屏接入后台
* 🛒 **[10. 综合实战：从零构建电商系统](../development/10-walkthrough-shop/)**：完整的端到端实战开发
* 🏆 **[11. 架构最佳实践与生产部署](../development/11-best-practices/)**：高并发调优与 Docker 容器化

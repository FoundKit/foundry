---
title: 快速入门
description: 基于 Git 依赖创建、构建并启动独立 Foundry 应用的 5 分钟上手指南，全面支持 PostgreSQL、MySQL、MariaDB 与 MongoDB。
---

# Foundry 快速入门

Foundry 是一个现代、模块化、完全解耦的 Rust 后端平台与开发框架。在当前测试与验证阶段（正式发布至 crates.io 前），开发者可以通过 **Git 依赖** 的方式在自己的独立仓库中引入并使用 Foundry。

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

脚手架会根据创建时选定的数据库，自动生成对应的特性依赖配置：

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
# --- Foundry 框架依赖与数据库特性切换 ---
# 选项 A: PostgreSQL (默认已启用，无需额外配置)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# 选项 B: MongoDB (Zero-DDL 原生文档引擎)
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mongodb"] }

# 选项 C: MySQL
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mysql"] }

# 选项 D: MariaDB
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mariadb"] }

# 选项 E: 全数据库支持 (同时编译所有引擎)
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["all-databases"] }

# --- 运行时与 Web 基础设施 ---
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

### 3. 启动并初始化数据库与缓存服务

Foundry 应用通过根目录下 `.env` 的 `DATABASE_URL` 与 `DATABASE_TYPE` 连接底层数据库，并使用 Redis 作为租户隔离缓存与分布式锁。

#### 方式一：使用 Docker 启动本地数据库容器

在本地快速体验时，可直接通过 Docker 启动所需的数据库与 Redis 7 容器：

```bash
# 1. 启动 Redis 7 缓存服务 (所有数据库引擎均需配套使用)
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

检查项目根目录下的 `.env` 配置，根据实际所使用的数据库进行配置：

```bash
HOST=0.0.0.0
PORT=8080

# -----------------------------------------------------------------------------
# 数据库配置（根据实际使用的数据库取消对应注释）
# -----------------------------------------------------------------------------

# 选项 A: PostgreSQL (默认)
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
DATABASE_TYPE=postgres

# 选项 B: MongoDB (Zero-DDL，需在 Cargo.toml 开启 features = ["standard", "mongodb"])
# DATABASE_URL=mongodb://127.0.0.1:27017/foundry
# DATABASE_TYPE=mongodb

# 选项 C: MySQL (需在 Cargo.toml 开启 features = ["standard", "mysql"])
# DATABASE_URL=mysql://root:root@localhost:3306/foundry
# DATABASE_TYPE=mysql

# 选项 D: MariaDB (需在 Cargo.toml 开启 features = ["standard", "mariadb"])
# DATABASE_URL=mariadb://root:root@localhost:3307/foundry
# DATABASE_TYPE=mariadb

# -----------------------------------------------------------------------------
# 缓存与安全配置
# -----------------------------------------------------------------------------
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production

# 服务启动时是否执行自动建表/创建集合与超管初始化
AUTO_MIGRATE=true
```

> [!IMPORTANT]
> **深入理解 AUTO_MIGRATE 与 DBA 模式**：
> - **当 `AUTO_MIGRATE=true` 时（开发与轻量生产默认推荐）**：
>   执行 `cargo run` 启动时，Foundry 会根据所配置的数据库自动完成初始化：
>   - **SQL 系列（PostgreSQL / MySQL / MariaDB）**：自动检查基础元数据表，按需创建缺失表、生成覆盖索引并注入默认超级管理员。
>   - **MongoDB**：自动创建全量集合（`records`, `models`, `systems`, `configs`, `admins`, `audit_logs`），创建唯一/通配符索引，并写入超管文档，全程 **无需任何 SQL DDL 迁移**。
> - **当 `AUTO_MIGRATE=false` 时（严格生产 DBA 审批审计模式）**：
>   可通过 CLI 导出离线初始化脚本：
>   - 关系型数据库：执行 `foundry ddl -d postgres`（或 `mysql` / `mariadb`）导出 `migrations/init.sql`。
>   - MongoDB：执行 `foundry ddl -d mongodb` 导出 `migrations/init.js` 和 `migrations/README.md`，供 DBA 通过 Mongo Shell 离线执行：
>     ```bash
>     mongosh "mongodb://127.0.0.1:27017/foundry" migrations/init.js
>     ```

---

### 4. 运行应用程序

在项目根目录下启动后端服务：

```bash
cargo run
```

服务启动时，Foundry 将全自动完成以下初始化流程：
1. 通过协议嗅探智能连接所配置的数据库（**PostgreSQL**、**MongoDB**、**MySQL** 或 **MariaDB**）与 Redis。
2. 自动检查数据库元数据与创建索引，并确保初始超级管理员账号就绪（`AUTO_MIGRATE=true`）。
3. 挂载内置的嵌入式 React 管理后台与所有业务子系统自定义路由。
4. 监听并在 `http://0.0.0.0:8080` 提供对外服务。

---

### 5. 打开后台管理控制台

在浏览器中直接访问根路径或管理路径（访问根路径将自动重定向至管理后台）：

```text
http://localhost:8080/
# 或
http://localhost:8080/admin/
```

#### 默认管理员账号与密码：
* **用户名**: `admin`
* **初始密码**: `admin123456`
* **角色**: `super_admin`（超级管理员）

#### 管理控制台核心能力：
* **管理员与 RBAC 权限管理**：添加多级管理员、重置密码、精细分配子系统访问权限（`allowed_systems`）。
* **Zero-DDL 动态数据建模中心**：在界面上直接查看和配置动态模型、字段结构与数据记录（在 PostgreSQL、MySQL、MariaDB 和 MongoDB 上获得完全一致的体验）。
* **子系统扩展页面**：通过内嵌 iframe 容器，完美融入各业务子系统自定义定制的 Admin 大屏看板（如示例工程中的 Sample 页面）。
* **平台系统配置与审计日志**：全局系统参数修改、实时审计与操作跟踪记录。

---

### 6. 验证接口调用

#### 1. 系统健康检查：
```bash
curl http://localhost:8080/api/v1/health
# 返回: OK
```

#### 2. 超管登录与 JWT 鉴权：
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123456"}'

# 返回:
# {
#   "code": 0,
#   "message": "success",
#   "data": {
#     "token": "eyJhbGciOiJIUzI1NiIsIn...",
#     "user": {
#       "username": "admin",
#       "role": "super_admin"
#     }
#   }
# }
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

### 7. 常用 CLI 研发与脚手架指令

在日常研发中，你可以使用 `foundry` CLI 提高开发效率：

```bash
# 在现有应用中新建业务子系统模块 (如 billing 计费中心、orders 订单中心)
foundry system new billing --name "计费中心"

# 扫描并列出当前工程中所有已发现的业务子系统
foundry system list

# 导出当前数据库方言建表脚本或 MongoDB 初始化脚本 (供 DBA 归档审计)
foundry ddl --db postgres    # 导出 migrations/init.sql
foundry ddl --db mongodb     # 导出 migrations/init.js

# 校验当前工程的项目结构、子系统规范与 Cargo 依赖完整性
foundry validate
```

---

## 🧩 下一步指南

- **[系统开发完全指南](../guides/system-development/)**：参考 ThinkPHP / NestJS 架构思维，掌握三层架构、链式 ORM 查询构造器与全流程实战。
- **[数据库与自定义存储开发](../guides/database/)**：深入学习 Multi-Database SPI、MongoDB BSON 原生存储、PostgreSQL GIN 索引与自定义 ORM 实体编写。
- **[子系统与自定义功能开发](../guides/extensions/)**：学习自定义控制器、生命周期钩子以及 Admin 扩展看板设计。
- **[CLI 命令行工具指南](../guides/cli/)**：全面掌握脚手架创建、子系统生成与运维命令。
- **[存储引擎架构设计](../architecture/storage-engines/)**：深入了解底层 SPI 设计蓝图与各数据库性能优化考量。

---
title: CLI 命令行工具指南
description: foundry 与 foundry-cli 开发者命令行工具完整参考手册。
---

# Foundry CLI 命令行工具指南

Foundry 提供了功能完备的开发者命令行工具，可同时通过 `foundry` 或 `foundry-cli` 命令调用。

---

## 📦 安装方式

```bash
# 方式 1: 直接从 GitHub 仓库安装 (测试与开发阶段)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 方式 2: 本地源码编译安装
cargo install --path crates/foundry_cli
```

---

## 🛠️ 命令概览

| 命令 | 说明 |
|---|---|
| `foundry new <name> [--db <type>]` | 创建全新的独立业务应用工程 |
| `foundry system new <slug>` | 在当前工程中脚手架生成业务子系统代码骨架 |
| `foundry system list` | 扫描并列出当前工程所有已注册子系统 |
| `foundry migrate` | 执行数据库基准表与业务迁移 |
| `foundry ddl --db <type>` | 生成特定数据库方言的 DDL 迁移脚本 |
| `foundry validate` | 校验当前工程与子系统的目录结构与规范完整性 |

---

## 1. 创建独立工程: `foundry new`

创建包含标准三层架构与 Git 依赖的独立 Rust 应用：

```bash
# 标准创建 (默认使用 PostgreSQL 与 Git 依赖指向 main 分支)
foundry new my-app

# 指定特定数据库引擎创建 (postgres, mysql, mariadb, mongodb)
foundry new my-app --db mysql
foundry new my-app --db mariadb
foundry new my-app --db mongodb

# 本地源码调试创建 (指定 local path 依赖)
foundry new my-app --path /path/to/foundry/crates/foundry

# 指定特定 Git 仓库地址或分支
foundry new my-app --git https://github.com/foundkit/foundry --branch feature/next

# 指定特定 crates.io 版本
foundry new my-app --version 0.1.0
```

> **数据库配置与纯净骨架**：通过 `foundry new --db <type>` 创建工程时，框架会生成纯净代码骨架（默认不生成 `migrations/` 目录），并在 `.env` 中预置匹配的 `DATABASE_URL` 与 `DATABASE_TYPE`。当 `AUTO_MIGRATE=true` 时，执行 `cargo run` 框架将直接通过内嵌预设 DDL 自动初始化库表、索引及超级管理员账号，无需物理 SQL 文件。若需要交由 DBA 审计或外部工具部署，可执行 `foundry ddl` 按需导出方言脚本。

---

## 2. 业务子系统管理: `foundry system`

### 生成业务子系统骨架: `foundry system new`

在当前工程的 `src/systems/<slug>/` 下快速生成业务子系统代码骨架：

```bash
foundry system new billing --name "账单与支付中心"
```

自动生成的文件结构包括：
* `src/systems/billing/mod.rs`（实现 `SubsystemModule` 特征）
* `src/systems/billing/controllers/mod.rs`（Axum HTTP 路由控制器）
* `src/systems/billing/logic/mod.rs`（纯领域业务逻辑与数据库交互）
* `src/systems/billing/dto/mod.rs`（请求入参与 validator 规则校验）
* `src/systems/billing/custom_pages/`（嵌入管理后台的专属运营看板）

### 查看已注册子系统: `foundry system list`

扫描并列出工程内所有代码内聚子系统：

```bash
foundry system list
```

---

## 3. 数据库迁移与 DDL 生成

### 执行数据库迁移: `foundry migrate`

手动对数据库执行基准迁移：
```bash
# 执行当前项目迁移（自动读取 .env 的 DATABASE_URL 与 DATABASE_TYPE）
foundry migrate

# 显式指定连接串与方言
foundry migrate --database-url postgres://postgres:postgrespassword@localhost:5432/foundry --database postgres

# 仅生成方言 DDL 到 migrations/ 目录，不连接数据库
foundry migrate --generate
```

> 💡 **原理说明**：`foundry migrate`（与应用启动时的 `AUTO_MIGRATE=true`）执行的是 Rust 库中**编译期预设内嵌的方言 DDL / 原生 API**，无需磁盘上存在物理 `migrations/` 目录。

### 生成方言 DDL: `foundry ddl`

`foundry ddl` 直接从编译期二进制 SPI 存储驱动（`foundry_storage`）读取基准方言 DDL，并自动合成所有通过代码注册的扩展 DDL。

为 DBA 审计或外部迁移工具导出特定数据库方言的初始化脚本（将在当前目录下创建 `migrations/` 目录）：
```bash
# 为当前项目生成（自动读取 .env 中的 DATABASE_TYPE）
foundry ddl

# 显式指定数据库方言
foundry ddl --db mysql        # 生成 migrations/init.sql
foundry ddl --db postgres     # 生成 migrations/init.sql
foundry ddl --db mariadb      # 生成 migrations/init.sql
foundry ddl --db mongodb      # 生成 migrations/init.js 与 migrations/README.md

# 导出到标准输出 (stdout) 或自定义文件
foundry ddl --db mysql -o -
foundry ddl --db postgres -o deploy/init.sql
```

#### Code-First ORM 架构与代码级 DDL 机制

Foundry 采用 **ORM 优先 (ORM-First, Prisma 风格)** 的现代化架构。对于支持关系型 ORM 的数据库（PostgreSQL, MySQL, MariaDB），`foundry migrate` 与 `foundry ddl` 均直接从二进制代码中的 ORM 模型编译生成 DDL：

1. **子系统代码级 ORM 模型声明（Prisma 风格，推荐）**：
   在实现 `SubsystemModule` 特征时，重写 `orm_models` 方法返回实体 Schema 定义：
   ```rust
   impl SubsystemModule for BillingSubsystem {
       fn slug(&self) -> &'static str { "billing" }
       fn display_name(&self) -> &'static str { "计费中心" }
       fn register_routes(&self, router: Router) -> Router { router }

       // 在代码中声明子系统 ORM 模型（类似 Prisma Schema）
       fn orm_models(&self) -> Vec<EntitySchema> {
           vec![
               EntitySchema::new("billing_invoices")
                   .column(ColumnDef::bigint("id").primary_key().auto_increment())
                   .column(ColumnDef::varchar("system_id", 32).not_null())
                   .column(ColumnDef::varchar("invoice_no", 64).not_null().unique())
                   .column(ColumnDef::decimal("amount", 10, 2).not_null())
                   .column(ColumnDef::varchar("status", 24).not_null())
                   .column(ColumnDef::datetime("created_at").default_current_timestamp())
                   .index(IndexDef::new("idx_invoices_sys", &["system_id"]))
           ]
       }
   }
   ```
   当通过 `app.register_subsystem(BillingSubsystem)` 注册时，框架自动收集这些 ORM 模型。
   执行 `foundry ddl --db postgres`（或 `mysql` / `mariadb`）以及执行迁移时，所有核心表与子系统 ORM 表将自动编译并同步！

2. **封装的 ORM 增删改查 (CRUD Query Builder)**：
   业务逻辑层读写数据库时，使用类型安全的 ORM 构建器执行 CRUD，无需手写 SQL 字符串，自动适配方言参数：
   ```rust
   // ORM Insert
   let (sql, binds) = OrmInsert::into_table("billing_invoices")
       .value("system_id", "billing")
       .value("invoice_no", "INV-001")
       .value("amount", 99.0)
       .to_sql(Dialect::Postgres);

   // ORM Select
   let (sql, binds) = OrmSelect::from_table("billing_invoices")
       .where_eq("system_id", "billing")
       .limit(10)
       .to_sql(Dialect::Postgres);
   ```

3. **原生 SQL 与特殊脚本逃生机制 (Fallback)**：
   若特定数据库无关系型 ORM（如 MongoDB），或需要特殊触发器、自定义扩展，可使用 `migration_ddl`：
   ```rust
   fn migration_ddl(&self, db_type: &str) -> Option<String> {
       match db_type {
           "postgres" => Some("CREATE INDEX IF NOT EXISTS idx_custom ON ...;".into()),
           _ => None,
       }
   }
   ```

4. **自定义数据库驱动扩展 (StorageDriverProvider)**：
   若需接入全新的数据库引擎（如 SQLite、CockroachDB 等），只需实现 `StorageDriverProvider`（实现 `get_baseline_ddl()`、`create_engines()` 与 `run_migrations()`），并通过 `foundry_storage::register_storage_provider(Arc::new(MyCustomProvider))` 注入二进制 SPI 注册表。

---

## 4. 项目校验: `foundry validate`

检测当前工程结构与已注册子系统元数据是否规范：

```bash
foundry validate
```

---

## 5. 后台管理员与权限管控说明

Foundry 遵循职责分离的最佳实践原则：
* **CLI 聚焦研发工具链**：专注于项目创建、代码骨架生成、数据库迁移与工程校验。
* **Web 后台负责运行期运维**：管理员账号的增删、角色分配（`super_admin` / `admin` / `topic_admin`）以及密码重置已全面内置在开箱即用的 React 管理后台（`/admin`）中。
* 系统初始化时会自动创建默认初始超管（`admin / admin123456`），后续运维直接登录后台进行可视化安全管理，无需通过终端明文传递敏感凭据。

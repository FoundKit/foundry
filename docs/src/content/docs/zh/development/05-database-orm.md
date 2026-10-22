---
title: 数据库配置与 Code-First ORM
description: 掌握 Foundry 多数据库 SPI 架构、Code-First 实体模型定义、链式查询构造器与数据库平滑迁移工作流。
---

本章内容：
1. 理解 Foundry 多数据库 SPI 插件化架构与连接配置；
2. 学会在子系统中通过代码优先（Code-First）定义实体表结构与索引；
3. 熟练运用链式 ORM 查询构造器进行增删改查；
4. 掌握软删除（Soft Delete）与全自动数据库迁移（Migrations）。

---

## 1. 多数据库 SPI 架构与连接配置

Foundry 采用现代的 **SPI (Service Provider Interface) 存储架构**，底层网络驱动彻底封装，对业务层零泄漏。统一通过根目录下的 `.env` 声明：

```bash
# .env

# 1. 数据库类型：postgres | mysql | mariadb | mongodb
DATABASE_TYPE=postgres

# 2. 数据库连接 URL
# PostgreSQL: postgres://user:password@localhost:5432/foundry
# MySQL:      mysql://user:password@localhost:3306/foundry
# MariaDB:    mariadb://user:password@localhost:3306/foundry
# MongoDB:    mongodb://localhost:27017/foundry
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry

# 3. 自动迁移开关 (开发/CI 环境设为 true，生产环境设为 false)
AUTO_MIGRATE=true
```

---

## 2. Code-First 实体模型定义 (Prisma 风格)

无需手写繁琐的 SQL DDL 脚本。子系统可以在 `SubsystemModule::orm_models()` 中直接以 Rust 代码声明数据表结构：

```rust
// src/systems/shop/mod.rs
use foundry::prelude::*;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str { "shop" }
    fn display_name(&self) -> &'static str { "电子商务商城" }

    // 代码定义 ORM 实体结构 (Code-First)
    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            // 订单主表
            EntitySchema::new("shop_orders")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("order_no", 64).not_null().unique())
                .column(ColumnDef::varchar("customer_name", 64).not_null())
                .column(ColumnDef::varchar("customer_email", 128).not_null())
                .column(ColumnDef::decimal("total_amount", 12, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null()) // pending, paid, shipped
                .column(ColumnDef::json("items_payload").default_empty_json())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("updated_at").default_current_timestamp())
                .column(ColumnDef::datetime("deleted_at").nullable()) // 软删除标记
                // 联合索引：必须以 system_id 开头以保证租户隔离性能
                .index(IndexDef::new("idx_orders_sys_status", &["system_id", "deleted_at", "status"])),

            // 商品表
            EntitySchema::new("shop_products")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("title", 128).not_null())
                .column(ColumnDef::decimal("price", 10, 2).not_null())
                .column(ColumnDef::integer("stock").not_null())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("deleted_at").nullable())
        ]
    }
}
```

> [!TIP]
> **脚手架一键生成**：
> 执行 `foundry system new <slug> --with-table --table-name <表名>`，CLI 将自动为你生成包含 `deleted_at` 软删除、ORM 实体定义、DTO、Service 以及全套 CRUD 控制器的完整子系统模板！

### 2.1 单子系统多表支持与后期平滑追加新表

很多开发者会问：*“一个子系统只能建一张表吗？如果业务运行很久后才需要新建表怎么办？”*

答案是：**天然支持任意多张表，且支持运行时后期随时追加新表！**

* `SubsystemModule::orm_models(&self) -> Vec<EntitySchema>` 返回的是动态数组 `Vec`，你可以在一个子系统里声明 1 张、5 张甚至几十张关联物理表（如订单主表、订单明细表、库存流水表等）。
* **后期平滑加表**：当子系统上线运行了数月后，若需要追加一张退款表 `shop_refunds`，直接在 `orm_models()` 的 `vec![]` 中追加一个新的 `EntitySchema::new("shop_refunds")...`。
* **开发环境自动同步**：在 `.env` 中开启 `AUTO_MIGRATE=true`，应用重新启动时，框架的库表核验机制会自动比对数据库，检测到缺失 `shop_refunds` 后会**自动执行 `CREATE TABLE IF NOT EXISTS` 完成无缝热建表**，无需任何手工干预！

### 2.2 字段增减与混合模式最佳实践 (Hybrid Schema)

CLI 或示例中生成的字段（如 `title`, `content`）仅仅是起步脚手架示范，**表的字段绝对不是固定的，你可以随时自由增减**：

1. **代码三步同步**：
   * 在 `orm_models()` 中增减 `.column(...)` 声明；
   * 在 `models/mod.rs` 实体结构体中同步增减强类型字段；
   * 在 `dto/mod.rs` 中增减接口入参/出参字段。
2. **混合模式最佳实践（Hybrid Schema）**：
   对于既想要独立物理表的高性能与严格约束，又希望部分字段能像动态模型一样灵活扩展的业务，推荐采用**“核心物理列 + 扩展 JSON 列”**模式：
   ```rust
   EntitySchema::new("shop_orders")
       // 核心高频检索与对账字段 -> 强类型物理列 + B+Tree 索引
       .column(ColumnDef::bigint("id").primary_key().auto_increment())
       .column(ColumnDef::varchar("order_no", 64).not_null().unique())
       .column(ColumnDef::decimal("total_amount", 12, 2).not_null())
       .column(ColumnDef::varchar("status", 24).not_null())
       .index(IndexDef::new("idx_orders_status", &["status", "created_at"]))
       // 未来随时可能变动的自定义属性 -> 预留一个 JSON 扩展列
       .column(ColumnDef::json("extra_metadata").default_empty_json())
       .column(ColumnDef::datetime("deleted_at").nullable())
   ```
   * 状态流转、订单检索、财务结算走物理列，享受极速索引；
   * 渠道标签、用户定制备注等临时属性存入 `extra_metadata`，随时写入任意新属性而无需改动表结构。

### 2.3 导出包含自定义业务实体的全量 DDL

全局命令 `foundry ddl -d mysql` 导出的是平台 7 张内置核心表。如果你在子系统代码中通过 `orm_models()` 声明了自定义物理表，有以下两种方式导出 DDL：

1. **通过 CLI 脚手架生成（推荐）**：执行 `foundry system new <slug> --with-table --db mysql` 时，CLI 会自动将该表的建表 SQL 编译输出到 `migrations/mysql/<slug>.sql`。
2. **应用内一键导出（包含核心表 + 所有子系统表）**：
   在应用中注册子系统后，调用 `compile_all_ddl` 即可导出完整合集 DDL：
   ```rust
   // 注册所有子系统后，一键导出完整方言 DDL (适用于 CI/CD 自动生成上线 SQL)
   let full_mysql_ddl = foundry_storage::compile_all_ddl(foundry_storage::Dialect::MySql);
   println!("{}", full_mysql_ddl);
   ```

---

## 3. 链式查询构造器 (ORM Query Builder)

Foundry 提供了完全自适应各数据库方言（自动替换 `$1` 与 `?` 参数占位符）的强类型链式构造器：

### 3.1 链式查询 (`OrmSelect`)

```rust
use foundry::prelude::*;

// 1. 链式分页查询 (自动过滤软删除)
let (sql, binds) = OrmSelect::from_table("shop_orders")
    .column("id")
    .column("order_no")
    .column("total_amount")
    .column("status")
    .column("created_at")
    .where_eq("system_id", "shop")
    .where_eq("status", "pending")
    .where_null("deleted_at")          // 软删除过滤
    .order_by_desc("created_at")
    .limit(20)
    .offset(0)
    .to_sql(Dialect::Postgres);        // 或 Dialect::MySql

// sql: SELECT id, order_no, total_amount, status, created_at FROM shop_orders 
//      WHERE system_id = $1 AND status = $2 AND deleted_at IS NULL 
//      ORDER BY created_at DESC LIMIT $3 OFFSET $4
```

### 3.2 链式插入 (`OrmInsert`)

```rust
let (insert_sql, binds) = OrmInsert::into_table("shop_orders")
    .value("system_id", "shop")
    .value("order_no", "ORD-20261015-001")
    .value("customer_name", "张三")
    .value("customer_email", "zhangsan@example.com")
    .value("total_amount", 299.00)
    .value("status", "pending")
    .to_sql(Dialect::Postgres);
```

### 3.3 链式更新 (`OrmUpdate`)

```rust
let (update_sql, binds) = OrmUpdate::into_table("shop_orders")
    .set("status", "paid")
    .set_now("updated_at")
    .where_eq("order_no", "ORD-20261015-001")
    .where_null("deleted_at")
    .to_sql(Dialect::Postgres);
```

### 3.4 链式软删除 (生产严禁物理 DELETE)

```rust
// 安全软删除：标记 deleted_at，零物理 DELETE 权限亦可平稳运行
let (soft_del_sql, binds) = OrmUpdate::into_table("shop_orders")
    .set_now("deleted_at")
    .where_eq("order_no", "ORD-20261015-001")
    .where_null("deleted_at")
    .to_sql(Dialect::Postgres);
```

---

## 4. 数据库迁移工作流 (Migrations)

Foundry 支持两种灵活的迁移策略：

```mermaid
flowchart LR
    Dev["开发 / CI 环境 (AUTO_MIGRATE=true)"] --> Run["运行 cargo run 自动建表与升级"]
    Prod["生产环境 (AUTO_MIGRATE=false)"] --> Export["foundry ddl --db postgres 导出 migrations/init.sql"]
    Export --> DBA["DBA 审计并通过上线平台平稳发布"]
```

### CLI 常用迁移命令

```bash
# 1. 导出当前项目的全量方言 DDL 脚本到 migrations/ 目录
foundry ddl --db postgres           # 导出 PostgreSQL 脚本 (migrations/init.sql)
foundry ddl --db mysql              # 导出 MySQL 脚本 (migrations/init.sql)
foundry ddl --db mongodb            # 导出 MongoDB 脚本 (migrations/init.js)

# 2. 检查或手动执行迁移
foundry migrate
foundry migrate --generate          # 自动生成 migrations 目录与初始方言文件
```

---

## 5. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 参数化绑定彻底杜绝 SQL 注入**：
> 永远不要通过 `format!("SELECT * FROM ... WHERE id = {}", id)` 拼接 SQL！始终使用 `OrmSelect`、`OrmInsert` 等构造器生成的参数绑定，或使用底层驱动的参数占位符。
> 
> **2. 必须包含租户过滤条件**：
> 在多系统架构中，任何针对实体表的查询与更新，必须带上 `.where_eq("system_id", &ctx.system_slug)`，确保各子系统之间数据严格隔离。

下一节：[06. Zero-DDL 动态数据模型与 Auto-CRUD](./06-dynamic-models/)

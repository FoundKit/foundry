---
title: 05. Database Setup & Code-First ORM
description: Master Foundry's multi-database SPI storage, Code-First ORM schema definitions, ThinkPHP-style fluent query builders, and database migrations.
---

# 05. Database Setup & Code-First ORM

> **Chapter Objectives**:
> 1. Understand Foundry's multi-database SPI plugin storage architecture;
> 2. Define relational schemas and indexes using Code-First `EntitySchema`;
> 3. Use the fluent `OrmSelect`, `OrmInsert`, `OrmUpdate`, and soft delete builders;
> 4. Manage migration workflows with `foundry ddl` and `foundry migrate`.

---

## 1. Multi-Database SPI Configuration

Configure your database connection in `.env`:

```bash
# postgres | mysql | mariadb | mongodb
DATABASE_TYPE=postgres
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
AUTO_MIGRATE=true
```

---

## 2. Code-First Entity Schemas

In `SubsystemModule::orm_models`:

```rust
use foundry::prelude::*;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str { "shop" }
    fn display_name(&self) -> &'static str { "Shop" }

    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            EntitySchema::new("shop_orders")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("order_no", 64).not_null().unique())
                .column(ColumnDef::varchar("customer_name", 64).not_null())
                .column(ColumnDef::decimal("total_amount", 12, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("deleted_at").nullable())
                .index(IndexDef::new("idx_orders_sys", &["system_id", "deleted_at"]))
        ]
    }
}
```

---

## 3. Fluent Query Builders (ThinkPHP Style)

```rust
use foundry::prelude::*;

// 1. Paginated Select with Soft Delete filter
let (sql, binds) = OrmSelect::from_table("shop_orders")
    .column("id")
    .column("order_no")
    .column("total_amount")
    .where_eq("system_id", "shop")
    .where_null("deleted_at")
    .order_by_desc("created_at")
    .limit(20)
    .offset(0)
    .to_sql(Dialect::Postgres);

// 2. Insert
let (insert_sql, binds) = OrmInsert::into_table("shop_orders")
    .value("system_id", "shop")
    .value("order_no", "ORD-20261015-001")
    .value("customer_name", "Alice")
    .value("total_amount", 299.00)
    .value("status", "pending")
    .to_sql(Dialect::Postgres);

// 3. Soft Delete Update
let (soft_del_sql, binds) = OrmUpdate::into_table("shop_orders")
    .set_now("deleted_at")
    .where_eq("order_no", "ORD-20261015-001")
    .to_sql(Dialect::Postgres);
```

Next chapter: [06. Zero-DDL Dynamic Models & Auto-CRUD](./06-dynamic-models/)

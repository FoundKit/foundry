---
title: Database & Storage Engine Guide
description: Foundry Multi-Database SPI architecture, PostgreSQL, MySQL, MariaDB, and MongoDB configurations, Zero-DDL dynamic models, and local development.
---

> [!TIP]
> **Complete System Development Tutorial**: For chainable ORM query builders (`OrmSelect`, `OrmInsert`, `OrmUpdate`, `OrmDelete`), Code-First entity schemas, and end-to-end practical walkthroughs, refer to [05. Database Setup & Code-First ORM](../development/05-database-orm/).

Foundry features a modern **Multi-Database SPI (Service Provider Interface) pluggable storage architecture**. Business layers, domain services, the AutoCRUD engine, and Axum handlers exclusively depend on the `Database` facade and standard store traits, **achieving absolute zero driver leakage**.

Foundry officially supports four enterprise-grade databases:
1. **PostgreSQL (14+)** - Relational engine with JSONB and GIN inverted indexing.
2. **MySQL (8.0+)** - Universal single-table engine with covering indexes and deferred join pagination.
3. **MariaDB (10.5+)** - Dedicated MariaDB dialect engine with fine-tuned InnoDB indexes.
4. **MongoDB (6.0+)** - Native BSON document engine with atomic `i64` sequence counters and dynamic Wildcard indexing.

*(Note: Oracle DB has been removed from the roadmap due to excessive runtime weight, heavy C client dependencies, and insufficient open-source Rust community maintenance.)*

---

## 1. Core Architecture & Design Principles

```mermaid
flowchart TD
    subgraph AppLayer ["Application Layer / AutoCRUD Engine / Controllers"]
        Handler["Axum Handler (Extension<AppState>)"]
        Call["Method Calls: db.records() / db.models() / db.configs() / db.systems()"]
        Handler --> Call
    end

    subgraph FacadeLayer ["Unified Facade & SPI (foundry_storage)"]
        Call --> Facade["Database Facade"]
        Facade --> Registry["StorageRegistry (Driver Routing)"]
    end

    subgraph Engines ["Driver Implementations (Compiled on Demand)"]
        Registry -->|"postgres:// / postgresql://"| PgEngine["PostgresProvider (JSONB + GIN Inverted Index)"]
        Registry -->|"mysql://"| MySqlEngine["MySqlProvider (Universal Single-Table + Covering Index + Deferred Join)"]
        Registry -->|"mariadb://"| MariaEngine["MariaDbProvider (Dedicated Dialect + Covering Index)"]
        Registry -->|"mongodb:// / mongodb+srv://"| MongoEngine["MongoProvider (Native BSON + Atomic Counter + Wildcard Index)"]
    end
```

### Key Highlights
1. **Method-Driven & Zero Driver Leakage**: Business code accesses storage exclusively via trait methods such as `state.db.records().create(...)` or `state.db.records().list(...)`. No driver-specific types (`PgPool`, `MySqlPool`, `mongodb::Database`) are exposed to user applications.
2. **Smart Protocol Sniffing**: At startup, Foundry parses the `DATABASE_URL` protocol scheme:
   - `postgres://` or `postgresql://` $\rightarrow$ activates PostgreSQL engine.
   - `mysql://` $\rightarrow$ activates MySQL engine.
   - `mariadb://` $\rightarrow$ activates MariaDB engine.
   - `mongodb://` or `mongodb+srv://` $\rightarrow$ activates MongoDB engine.
   - Can also be explicitly overridden using `DATABASE_TYPE=postgres|mysql|mariadb|mongodb`.
3. **Conditional Compilation**: Database drivers are gated by Cargo feature flags to keep binary sizes lean and compile times fast.

---

## 2. Environment Setup & Feature Flags

### Cargo.toml Dependencies

Foundry defaults to `postgres`. Enable the corresponding feature flags in your `Cargo.toml` depending on your target database:

```toml
[dependencies]
# 1. PostgreSQL only (default)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# 2. Enable MySQL support
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mysql"] }

# 3. Enable MariaDB support
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mariadb"] }

# 4. Enable MongoDB support
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mongodb"] }

# 5. Enable all database engines simultaneously
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["all-databases"] }
```

### Environment Configuration (`.env`)

#### For PostgreSQL:
```bash
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry_dev
DATABASE_TYPE=postgres
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### For MySQL:
```bash
DATABASE_URL=mysql://root:root@localhost:3306/foundry_dev
DATABASE_TYPE=mysql
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### For MariaDB:
```bash
DATABASE_URL=mariadb://root:root@127.0.0.1:3307/foundry_dev
DATABASE_TYPE=mariadb
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### For MongoDB:
```bash
DATABASE_URL=mongodb://127.0.0.1:27017/foundry_dev
DATABASE_TYPE=mongodb
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

> **Note**: When `AUTO_MIGRATE=true`, Foundry will automatically verify that the database exists (creating it if absent for SQL engines or creating default collections and indexes for MongoDB) and execute migrations (schema, covering indexes, and seed superadmin account). When `AUTO_MIGRATE=false`, Foundry strictly validates baseline schema completeness during server bootstrap, refusing to start if required tables/collections are missing (preflight verification can also be run via `foundry migrate --check`).

---

## 3. Containerized Setup (Docker / Podman)

Applications scaffolded via `foundry new` include container configurations. You can run any database using standard Docker containers:

```bash
# PostgreSQL
docker run -d --name foundry-postgres -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry_dev postgres:16-alpine

# MySQL
docker run -d --name foundry-mysql -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry_dev mysql:8.4

# MariaDB
docker run -d --name foundry-mariadb -p 3307:3306 \
  -e MARIADB_ROOT_PASSWORD=root -e MARIADB_DATABASE=foundry_dev mariadb:11.4

# MongoDB
docker run -d --name foundry-mongodb -p 27017:27017 \
  mongo:7.0

# Redis
docker run -d --name foundry-redis -p 6379:6379 redis:alpine
```

---

## 4. Using the `Database` Facade in Application Code

Inject `AppState` into your Axum handlers to interact with storage engines without writing dialect-specific code:

```rust
use axum::{extract::Extension, Json};
use foundry::prelude::*;
use serde_json::json;

pub async fn create_article_handler(
    Extension(state): Extension<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // Create a dynamic record via Zero-DDL model engine
    let record = state
        .db
        .records()
        .create(
            &ctx.system_slug,
            "articles",
            json!({
                "title": payload.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled"),
                "content": payload.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                "views": 0,
                "is_published": true
            }),
        )
        .await?;

    Ok(Json(ApiResponse::success(json!({
        "id": record.id,
        "data": record.data
    }))))
}
```

### Store Engines Overview

| Domain Store | Accessor | Key Methods |
|---|---|---|
| **Dynamic Records** | `db.records()` | `create()`, `get_by_id()`, `update()`, `delete()`, `list()` |
| **Model Schemas** | `db.models()` | `list_models()`, `get_model()`, `create_model()`, `list_fields()`, `add_field()` |
| **Subsystems / Tenants** | `db.systems()` | `list()`, `list_paginated()`, `get_by_id()`, `get_by_slug()`, `create()`, `update()` |
| **System Configurations** | `db.configs()` | `get()`, `set()`, `get_aggregated()`, `get_schema()` |
| **Admins & RBAC** | `db.admins()` | `get_by_id()`, `get_by_username()`, `create()`, `update()` |
| **Audit Logs** | `db.audit()` | `insert()`, `list()` |

---

## 5. Engine Architecture & Performance Strategies

### 5.1 PostgreSQL Engine
- **Storage Strategy**: Relational tables with native `JSONB` document storage.
- **Indexing**: GIN inverted index (`CREATE INDEX ... USING gin (data)`) enables sub-millisecond query performance on arbitrary nested JSON fields.
- **Consistency**: Uses atomic `INSERT ... RETURNING *` for instant record hydration.

### 5.2 MySQL Universal Single-Table Engine
- **Covering Index**:
  ```sql
  INDEX idx_model_query (model_id, deleted_at, created_at DESC, id DESC),
  INDEX idx_system_query (system_id, deleted_at)
  ```
- **Deferred Join Pagination**: Scans only the covering index to fetch matching primary keys first, then joins back to retrieve the `data` JSON payload. Eliminates disk I/O bottlenecks during deep pagination.
- **Atomic Re-Fetch**: Atomically orchestrates `INSERT` $\rightarrow$ `last_insert_id()` $\rightarrow$ `SELECT` in a single transaction to maintain consistency across engines.

### 5.3 MariaDB Engine
- **Protocol Adaptation**: Accepts `mariadb://` connection strings with automated parameter normalization.
- **Dialect Optimization**: Tailored DDL migration with UTF8MB4 collations and high-concurrency InnoDB row formats.
- **Performance**: Shares the deferred join pagination architecture while leveraging MariaDB-specific JSON functional indexes.

### 5.4 MongoDB Native Document Engine
- **Storage Strategy**: Native BSON documents stored in collection `model_records`.
- **Atomic `i64` Sequence Counter**: Uses a dedicated `_counters` collection with `findOneAndUpdate` (`$inc`) to provide strictly increasing 64-bit integer IDs, guaranteeing 100% contract compatibility with SQL engines.
- **Wildcard Indexing**: Automatically builds a wildcard index on dynamic payload attributes:
  ```javascript
  db.model_records.createIndex({ "data.$**": 1 })
  ```
  This allows queries filtering on dynamic model fields (e.g. `data.category = "tech"`) to leverage index acceleration out of the box.
- **Native Document Filtering**: Translates pagination, soft-deletion (`deleted_at: null`), and tenancy scopes into native MongoDB aggregation and query filters.

---

## 6. Typed Configurations & Custom Migrations

### Strongly Typed System Configurations
You can deserialize dynamic configurations directly into Rust structs:

```rust
#[derive(Debug, serde::Deserialize)]
pub struct PaymentConfig {
    pub api_key: String,
    pub merchant_id: String,
    pub sandbox: bool,
}

// In handler:
let pay_cfg: Option<PaymentConfig> = state
    .db
    .get_typed_config(&ctx.system_slug, "payment_settings")
    .await?;
```

## 7. Subsystem Physical Table Partitioning & DDL Delivery (MySQL & MariaDB)

### 7.1 Why MySQL / MariaDB Require Dedicated Physical Tables

When managing numerous business subsystems and high data volumes, the physical characteristics of different database engines dictate contrasting architectural strategies:

```mermaid
flowchart TD
    DBSelection["Database Selection & Storage Strategy"]
    DBSelection --> Mongo["MongoDB"]
    DBSelection --> PG["PostgreSQL"]
    DBSelection --> MySQL["MySQL / MariaDB"]

    Mongo --> MongoStrategy["[Pure 0-DDL]<br/>Native BSON documents, collections created on-demand<br/>Wildcard indexing, no schema maintenance needed"]
    PG --> PGStrategy["[0-DDL Preferred]<br/>Native JSONB + GIN inverted index + partition tables<br/>Single universal table maintains stellar performance"]
    MySQL --> MySQLStrategy["[Dedicated Tables + Explicit DDL]<br/>No native GIN index, JSON query performance degrades<br/>Dedicated table per subsystem + typed ORM recommended"]
```

* **MongoDB**: A native document database (schemaless). Collections are automatically created upon inserting the first document, requiring zero DDL files.
* **PostgreSQL**: In Mode A (0-DDL universal model), binary `JSONB`, `GIN` inverted indexing, and partial indexes allow a single shared table to gracefully scale across hundreds of topics.
* **MySQL / MariaDB Single-Table Bottlenecks**:
  1. MySQL treats JSON columns essentially as BLOB storage without true GIN indexes, resulting in higher query overhead for dynamic properties.
  2. Storing tens of millions of rows across diverse subsystems within a single shared `records` table increases InnoDB B+Tree depth, heightens row-lock contention, and exerts heavy pressure on the Buffer Pool.
  3. **Best Practice**: For MySQL and MariaDB, **use `foundry system new <slug> --with-table` to partition each subsystem into its own dedicated physical table** with generated DDL SQL scripts.

### 7.2 Development Auto-Sync vs. Production DBA Execution

Foundry enforces a structured delivery contract for DDL across environments:

1. **Development & Testing (Fully Automated)**:
   Running `foundry migrate` executes the platform baseline schema and **automatically scans `migrations/<dialect>/*.sql`** to apply all subsystem incremental DDL scripts sequentially.
2. **Production Environments (DBA Offline Execution)**:
   In production, application database accounts typically lack `DDL` permissions (`CREATE TABLE`, `ALTER TABLE`, `DROP TABLE`).
   Generated `migrations/<dialect>/<slug>.sql` scripts include header audit comments and deployment guides, allowing DBAs to review and manually execute the DDL offline prior to application rollout.

---

## 8. System-Wide Strict Soft Delete (Zero-DELETE Privilege Guarantee)

### 8.1 Core Architectural Principle

Data is the most valuable asset in any enterprise. Foundry enforces a strict **Soft Delete** standard across all platform and business domains:

> [!IMPORTANT]
> **Zero Physical DELETE Guarantee**: All deletion operations across every platform feature (subsystems, business records, models, field definitions, configs, and admin accounts) **strictly update `deleted_at = NOW()`**. The storage engine never executes physical `DELETE FROM ...` SQL statements!

### 8.2 Entity Coverage & Soft Delete Conventions

Every core table and generated subsystem table follows these conventions:

| Domain | Physical Table | Soft Delete Column & Query Rules |
|---|---|---|
| **Subsystems / Topics** | `systems` | `deleted_at: Option<DateTime<Utc>>`, queries filter `WHERE deleted_at IS NULL` |
| **System Configs** | `system_configs` | `deleted_at: Option<DateTime<Utc>>`, config retrieval excludes soft-deleted items |
| **Model & Field Schemas** | `models`, `model_fields` | `deleted_at: Option<DateTime<Utc>>`, validation automatically skips soft-deleted fields |
| **Admin Accounts** | `admins` | `deleted_at: Option<DateTime<Utc>>`, authentication blocks soft-deleted admins |
| **Dynamic Records** | `model_records` / `record_values` | `deleted_at: Option<DateTime<Utc>>`, built-in filtering on pagination and searches |
| **Dedicated Subsystem Tables** | `<slug>_items` (via `--with-table`) | Preset `deleted_at DATETIME(6) NULL`, companion services use `set_now("deleted_at")` |

### 8.3 Principle of Least Privilege for Database Accounts

Because the entire platform operates purely on soft deletion, **even if DBAs revoke physical `DELETE` permissions from the application database account (granting only `SELECT, INSERT, UPDATE`), every feature of Foundry functions with 100% reliability!**

Key benefits:
1. **Accidental & Malicious Data Loss Prevention**: Even in worst-case credential leaks or logic anomalies, the database engine physically rejects destructive `DELETE FROM` statements.
2. **Audit Trails & Instant Recovery**: Deletions remain completely auditable, and records can be restored in seconds simply by resetting `deleted_at` to `NULL`.

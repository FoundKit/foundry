---
title: Database & Storage Engine Guide
description: Foundry Multi-Database SPI architecture, PostgreSQL and MySQL configurations, Zero-DDL dynamic models, and containerized development.
---

# Database & Storage Engine Guide

Foundry features a modern **Multi-Database SPI (Service Provider Interface) pluggable storage architecture**. Business layers, domain services, the AutoCRUD engine, and Axum state exclusively depend on the `Database` facade and standard store traits, **achieving absolute zero driver leakage**.

The platform provides first-class support for **PostgreSQL (14+)** and **MySQL (8.0+) / MariaDB (10.5+)**, with automatic protocol sniffing based on `DATABASE_URL`.

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
        Registry -->|"mysql:// / mariadb://"| MySqlEngine["MySqlProvider (Universal Single-Table + Covering Index + Deferred Join)"]
    end
```

### Key Highlights
1. **Method-Driven & Zero Driver Leakage**: Business code accesses storage exclusively via trait methods such as `state.db.records().create(...)` or `state.db.records().list(...)`. No `PgPool` or `MySqlPool` types are exposed.
2. **Smart Protocol Sniffing**: At startup, Foundry parses the `DATABASE_URL` protocol scheme:
   - `postgres://` or `postgresql://` $\rightarrow$ activates PostgreSQL engine
   - `mysql://` or `mariadb://` $\rightarrow$ activates MySQL / MariaDB engine
   - Can also be explicitly overridden using `DATABASE_TYPE=mysql` or `DATABASE_TYPE=postgres`.
3. **Conditional Compilation**: Database drivers are gated by Cargo feature flags to keep binary sizes lean.

---

## 2. Environment Setup & Feature Flags

### Cargo.toml Dependencies

Foundry defaults to `postgres`. To use MySQL, enable the `mysql` feature in your `Cargo.toml`:

```toml
[dependencies]
# PostgreSQL only (default)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# Enable MySQL / MariaDB support
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["mysql"] }

# Support both PostgreSQL and MySQL concurrently
foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["postgres", "mysql"] }
```

### Environment Configuration (`.env`)

#### For PostgreSQL:
```bash
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

#### For MySQL / MariaDB:
```bash
DATABASE_URL=mysql://root:root@localhost:3306/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

> **Note**: When `AUTO_MIGRATE=true`, Foundry will automatically verify that the database exists (creating it if absent) and execute dialect-specific migrations (schema, covering indexes, and seed superadmin account).

---

## 3. Containerized Setup (Docker / Nerdctl / Podman)

The project includes a standard `compose.yml` compatible with **Docker Compose**, **Nerdctl (containerd)**, and **Podman Compose**:

```bash
# Docker Compose
docker compose up -d postgres mysql redis

# Nerdctl (containerd)
nerdctl compose up -d postgres mysql redis

# Podman Compose
podman-compose up -d postgres mysql redis
```

Ports:
- **MySQL 8.4**: Port `3306`, user `root`, password `root`, database `foundry`.
- **PostgreSQL 18.6**: Port `5432`, user `postgres`, password `postgrespassword`, database `foundry`.
- **Redis 8.0**: Port `6379`.

---

## 4. Using the `Database` Facade in Application Code

Inject the `AppState` into your Axum handlers to interact with storage engines:

```rust
use axum::{extract::Extension, Json};
use foundry::prelude::*;
use serde_json::json;

pub async fn create_article_handler(
    Extension(state): Extension<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 1. Create a dynamic record via Zero-DDL model engine
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

### Store Engines Overview:

| Domain Store | Accessor | Key Methods |
|---|---|---|
| **Dynamic Records** | `db.records()` | `create()`, `get_by_id()`, `update()`, `delete()`, `list()` |
| **Model Schemas** | `db.models()` | `list_models()`, `get_model()`, `create_model()`, `list_fields()`, `add_field()` |
| **Subsystems / Tenants** | `db.systems()` | `list()`, `list_paginated()`, `get_by_id()`, `get_by_slug()`, `create()`, `update()` |
| **System Configurations** | `db.configs()` | `get()`, `set()`, `get_aggregated()`, `get_schema()` |
| **Admins & RBAC** | `db.admins()` | `get_by_id()`, `get_by_username()`, `create()`, `update()` |
| **Audit Logs** | `db.audit()` | `insert()`, `list()` |

---

## 5. MySQL Universal Single-Table Engine & Performance Tuning

For MySQL/MariaDB environments, Foundry implements a **Universal Single-Table Engine**:

### 5.1 Composite Covering Index
```sql
CREATE TABLE IF NOT EXISTS model_records (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    model_id BIGINT NOT NULL,
    system_id VARCHAR(32) NOT NULL,
    model_slug VARCHAR(48) NOT NULL,
    data JSON NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_model_query (model_id, deleted_at, created_at DESC, id DESC),
    INDEX idx_system_query (system_id, deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### 5.2 Deferred Join Pagination
Deep pagination (e.g. `OFFSET 100000 LIMIT 20`) on large JSON tables typically causes severe disk IO. `MySqlRecordEngine` uses **Deferred Join**:
1. Scan the index alone to obtain matching primary keys.
2. Join back on the primary key to retrieve the full `data` payload.

### 5.3 Atomic Re-Fetch on Insert
MySQL lacks the `RETURNING` clause. Foundry atomically orchestrates `INSERT` $\rightarrow$ `last_insert_id()` $\rightarrow$ `SELECT` in a single transaction, delivering strict consistency across all database engines.

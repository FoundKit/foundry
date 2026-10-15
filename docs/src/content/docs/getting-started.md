---
title: Getting Started
description: 5-minute quickstart guide to create, build, and run an independent Foundry application with PostgreSQL, MySQL, MariaDB, and MongoDB.
---

# Foundry Getting Started

> **Beginner-Friendly Guide**: Follow these simple steps to set up your environment, initialize a new project, write your first custom business endpoint, and launch your server in **less than 5 minutes**!

Foundry is a modern, decoupled Rust backend platform and framework. During initial development, projects consume Foundry via **Git dependencies** in standalone repositories.

```mermaid
flowchart LR
    Step1["1. Install CLI Tool"] --> Step2["2. Create Standalone App"]
    Step2 --> Step3["3. Start Database"]
    Step3 --> Step4["4. Launch App & Admin"]
    Step4 --> Step5["5. Write First API"]
```

> [!NOTE]
> **Enterprise Multi-Database SPI Support**: Foundry provides zero-driver-leakage abstractions over:
> 1. **PostgreSQL (14+)** *(Default)* — Relational storage with native JSONB and GIN inverted index acceleration.
> 2. **MongoDB (6.0+)** — Native BSON document engine with atomic counter sequences and wildcard dynamic indexing (Zero-DDL).
> 3. **MySQL (8.0+)** — General relational engine with covering indexes and deferred join pagination.
> 4. **MariaDB (10.5+)** — Dedicated MariaDB dialect with tuned InnoDB indexes.

---

## ⚡ 5-Minute Quickstart

### 1. Install Foundry CLI

Install the `foundry` CLI directly via Cargo:

```bash
cargo install --git https://github.com/foundkit/foundry foundry_cli
```

Verify the installation:

```bash
foundry --help
```

---

### 2. Create a Standalone Application

Use `foundry new` to generate a clean application. Choose your database with `--db`:

```bash
# Default (PostgreSQL)
foundry new my-app

# MongoDB (Zero-DDL schema-free document engine)
foundry new my-app --db mongodb

# MySQL (8.0+)
foundry new my-app --db mysql

# MariaDB (10.5+)
foundry new my-app --db mariadb

cd my-app
```

#### Generated `Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }
tokio = { version = "1.44", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

---

### 3. Start Database & Redis

Using Docker for rapid local development:

```bash
# 1. Start Redis 7
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# 2. Start PostgreSQL 18 (Default)
docker run -d --name my-postgres -p 5432:5432 \
  -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry \
  postgres:18-alpine
```

Check `.env`:
```bash
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
DATABASE_TYPE=postgres
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

---

### 4. Run the Application

```bash
cargo run
```

The application will:
1. Connect to the database and Redis.
2. Auto-migrate tables/collections and inject the default superadmin (`AUTO_MIGRATE=true`).
3. Mount the embedded React Admin console and all subsystem routes.
4. Listen on `http://0.0.0.0:8080`.

---

### 5. Open the Admin Console

Visit:
```text
http://localhost:8080/admin/
```
* **Username**: `admin`
* **Password**: `admin123456`
* **Role**: `super_admin`

---

### 6. Verify Endpoints

```bash
# Health check
curl http://localhost:8080/health

# Sample subsystem greeting
curl -X POST http://localhost:8080/api/v1/s/sample/ext/greet \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'
```

---

## 🛠️ 3-Minute Practical: Writing Your First Custom API

Let's add a calculator endpoint to the `sample` subsystem:

### Step 1: Add the handler in `src/systems/sample/controllers/mod.rs`

```rust
use foundry::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    pub a: f64,
    pub b: f64,
    pub op: String,
}

pub async fn handle_calc(
    Json(payload): Json<CalcRequest>,
) -> AppResult<ApiResponse<f64>> {
    let result = match payload.op.as_str() {
        "add" => payload.a + payload.b,
        "sub" => payload.a - payload.b,
        "mul" => payload.a * payload.b,
        "div" => {
            if payload.b == 0.0 {
                return Err(AppError::BadRequest("Division by zero".into()));
            }
            payload.a / payload.b
        }
        _ => return Err(AppError::BadRequest("Unknown operator".into())),
    };

    Ok(ApiResponse::ok(result))
}
```

### Step 2: Bind the route

```rust
pub fn build_routes() -> Router {
    Router::new()
        .route("/greet", post(handle_greet))
        .route("/calc", post(handle_calc)) // Add this line
}
```

### Step 3: Test with curl

```bash
curl -X POST http://localhost:8080/api/v1/s/sample/ext/calc \
  -H "Content-Type: application/json" \
  -d '{"a": 100, "b": 25, "op": "div"}'

# {"code":0,"message":"success","data":4.0,"meta":null}
```

---

## ❓ Beginner FAQs

> [!TIP]
> **Can I run without Redis installed?**
> Yes! Leave `REDIS_URL=` blank in `.env`, and Foundry will gracefully degrade to in-memory mode without crashing.
>
> **Where are the core types imported from?**
> A single `use foundry::prelude::*;` re-exports 95% of daily web types (`Router`, `Json`, `Query`, `Path`, `ApiResponse`, `AppResult`, `info!`, etc.).

---

## 🧩 Next Steps (ThinkPHP-Inspired Modular Tutorial)

* 🏗️ **[01. Architecture & Lifecycle](../development/01-architecture-lifecycle/)**
* 🛣️ **[02. Routing System & Namespaces](../development/02-routing/)**
* 🎮 **[03. Controllers & Response Envelope](../development/03-controllers/)**
* 🛡️ **[04. Request Validation & DTOs](../development/04-validation/)**
* 💾 **[05. Database Setup & Code-First ORM](../development/05-database-orm/)**
* ⚡ **[06. Zero-DDL Dynamic Models & Auto-CRUD](../development/06-dynamic-models/)**
* 🔐 **[07. Authentication & Hierarchical RBAC](../development/07-auth-security/)**
* 🪝 **[08. Lifecycle Mutation Hooks](../development/08-mutation-hooks/)**
* 🖥️ **[09. Admin Console & postMessage SDK](../development/09-admin-integration/)**
* 🛒 **[10. Hands-on Walkthrough: E-Commerce System](../development/10-walkthrough-shop/)**
* 🏆 **[11. Architectural Best Practices & Production](../development/11-best-practices/)**

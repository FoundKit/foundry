# Foundry

<div align="center">

**A modern, modular, decoupled Rust backend platform and application framework.**

*Build, isolate, and extend independent backend systems and admin control planes with zero upstream coupling.*

<p align="center">
  <b>English</b> | <a href="README_zh.md">简体中文</a>
</p>

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-16%2B%20%2F%2018%2B-336791.svg)](https://www.postgresql.org)
[![MySQL](https://img.shields.io/badge/mysql-8.0%2B%20%2F%208.4%2B-4479A1.svg)](https://www.mysql.com)
[![Redis](https://img.shields.io/badge/redis-7%2B%20%2F%208-dc382d.svg)](https://redis.io)

</div>

---

## 🧭 Overview & Core Positioning

Foundry is a **Rust Backend Platform & Framework** designed for building robust, multi-system backend services with an embedded administrative control plane.

Unlike traditional boilerplate templates, Foundry separates the **Framework/Platform** from the **User Application**:

```text
Foundry Repository (Framework)
    │
    ├── crates/foundry             # Main facade crate (FoundryApp, Builder, prelude)
    ├── crates/foundry_core        # Context, errors, responses, Subsystem trait
    ├── crates/foundry_storage     # Zero-DDL dynamic models, configs, Multi-DB SPI (PostgreSQL, MySQL), Redis
    ├── crates/foundry_auth        # Admin IAM, Argon2id, JWT, Topic-scoped RBAC
    ├── crates/foundry_engine      # Multi-system router, Auto-CRUD, Audit middleware
    ├── crates/foundry_extension   # Mutation hooks & extension pipeline
    ├── crates/foundry_cli         # `foundry` and `foundry-cli` tooling
    ├── apps/admin                 # Decoupled React + Tailwind visual Admin Shell
    └── examples/blog_platform     # Standalone application reference
            │
            │ cargo publish
            ↓
    Foundry Cargo Crates
            │
            ↓
User Application (Independent Git Repo)
    ├── Cargo.toml                 # [dependencies] foundry = "0.1" (configured with chosen DB feature)
    ├── .env                       # Environment configuration (.env.example included)
    ├── src/
    │   ├── main.rs                # Bootstrap with FoundryApp::builder()
    │   └── systems/               # User business subsystems
    │       ├── blog/              # Controllers + Domain Logic + DTOs + Admin Views
    │       └── newsletter/
    └── migrations/                # (Optional) On-demand generated via `foundry ddl` or `foundry migrate -g`
        └── init.sql               # Platform schema & initial superadmin seed (init.js for MongoDB)
```

---

## ⚡ Developer Experience

Creating and running a new Foundry project takes seconds:

```bash
# 1. Install Foundry CLI (from GitHub repository)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 2. Scaffold a brand new standalone application (PostgreSQL by default, or --db mysql / mariadb / mongodb)
foundry new my-app --db postgres
cd my-app

# 3. Start local development database (or connect to existing database configured in .env)
# Example using Docker:
docker run -d --name my-postgres -p 5432:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry postgres:18-alpine
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# 4. Start the server (auto-initializes schema, indexes, and superadmin when AUTO_MIGRATE=true)
cargo run
```

Your generated application is completely independent and consumes Foundry through standard Cargo dependencies:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
foundry = "0.1.0"
tokio = { version = "1.44", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FoundryConfig::from_env();

    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(SampleSubsystem)
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

---

## 🌟 Key Architectural Features

1. **Framework & Application Total Decoupling**: Application developers build inside their own Git repository. Upgrading Foundry requires only `cargo update` with zero upstream git merge conflicts.
2. **High-Level Facade (`foundry`) & Public APIs**: Intuitive `FoundryApp::builder()` API, type-safe error envelopes (`AppError`, `AppResult`), standardized response wrappers (`ApiResponse`), and rich re-exports in `foundry::prelude::*`.
3. **Subsystem Architecture**: Organize business domains into self-contained subsystems implementing `SubsystemModule` (controllers, logic services, validation DTOs, and custom admin views).
4. **Zero-DDL Dynamic Storage & Multi-Database SPI**: Define system configs and dynamic data models visually without DDL migrations. Built-in pluggable support for PostgreSQL, MySQL 8.0+, MariaDB 10.5+, and MongoDB 6.0+ via clean SPI plugin architecture.
5. **Instant RESTful Auto-CRUD**: Dynamic data models automatically receive REST endpoints (`/api/v1/s/{system_slug}/{model_slug}`) with pagination and search across all supported databases.
6. **Decoupled Admin Shell & SDK Bridge**: The Admin SPA (`apps/admin`) embeds subsystem custom pages (HTML, React, Vue) inside sandboxed views, injecting JWT authentication and theme settings automatically via `postMessage` protocol.
7. **Hierarchical RBAC**: Fine-grained role-based access control with Super Admins (`super_admin`), Platform Admins (`admin`), and Topic-Scoped Admins (`topic_admin`).
8. **Lifecycle Mutation Hooks**: Intercept entity lifecycle events (`before_create`, `after_create`, `before_update`, `after_update`, `before_delete`, `after_delete`) using `MutationHook`.

---

## 📦 Workspace Crates

| Crate | Description |
|---|---|
| [`foundry`](crates/foundry) | Top-level facade crate providing `FoundryApp`, `FoundryBuilder`, and `prelude`. |
| [`foundry_core`](crates/foundry_core) | Core primitives, `SystemContext`, `SubsystemModule`, `AppError`, and response models. |
| [`foundry_storage`](crates/foundry_storage) | Multi-Database SPI (PostgreSQL, MySQL, MariaDB, MongoDB), Redis cache, Zero-DDL dynamic models, and migrations. |
| [`foundry_auth`](crates/foundry_auth) | Admin identities, Argon2id hashing, JWT token service, and Topic RBAC guards. |
| [`foundry_engine`](crates/foundry_engine) | Unified Axum routing, Auto-CRUD handler engine, and audit logging middleware. |
| [`foundry_extension`](crates/foundry_extension) | Lifecycle mutation hook pipelines and extension mechanisms. |
| [`foundry_cli`](crates/foundry_cli) | Developer CLI binaries (`foundry` & `foundry-cli`) for project and subsystem scaffolding. |

---

## 🛠️ CLI Commands & Core Workflows

Foundry provides a streamlined developer CLI (`foundry` / `foundry-cli`) to manage applications, subsystems, and models.

### 1. Database Setup & Multi-Engine CLI

Foundry connects to your database via `DATABASE_URL` and `DATABASE_TYPE` in `.env`. Choose your database during scaffolding or generate DDL on demand:

```bash
# Scaffold application for a specific database (pure code skeleton, migrations generated on-demand):
foundry new my-app --db postgres    # PostgreSQL (default)
foundry new my-app --db mysql       # MySQL 8.0+
foundry new my-app --db mariadb     # MariaDB 10.5+
foundry new my-app --db mongodb     # MongoDB 6.0+

# Generate dialect-specific DDL scripts anytime (creates migrations/):
foundry ddl --db postgres           # Writes PostgreSQL schema to migrations/init.sql
foundry ddl --db mysql              # Writes MySQL schema to migrations/init.sql
foundry ddl --db mongodb            # Writes MongoDB init.js to migrations/init.js

# Apply migrations manually via CLI (or generate with --generate):
foundry migrate
foundry migrate --generate          # Generates migrations/ for current project database
```

* **Zero-Setup Startup (`AUTO_MIGRATE=true`)**: Running `cargo run` automatically initializes tables/collections, covering indexes, and superadmin seed (`admin` / `admin123456`).
* **Strict DBA Auditing (`AUTO_MIGRATE=false`)**: Execute generated migration scripts directly (`psql -f migrations/init.sql`, `mysql < migrations/init.sql`, etc.).

### 2. Subsystem (Subproject) Operations

Organize business capabilities into high-cohesion, isolated subsystems:

```bash
# 1. Create a code-first subsystem inside src/systems/<slug>/
foundry system new billing --name "Billing Center"

# 2. Register it in src/systems/mod.rs:
#    pub mod billing;
#    pub use billing::BillingSubsystem;

# 3. Register it in src/main.rs:
#    .register_subsystem(BillingSubsystem)

# 4. List all discovered subsystems
foundry system list
```

### 3. Data Model Creation & Storage

Foundry supports an **ORM-First (Prisma-like)** architecture along with flexible multi-paradigm storage models:

#### Option A: Code-First ORM Models (Prisma-like Strong Typing, Recommended)
Define strongly-typed ORM schemas directly in subsystem code without manually writing raw SQL dialect migrations. When running `foundry migrate` or `foundry ddl`, Foundry compiles the code-defined ORM schemas into target database dialect DDL (PostgreSQL, MySQL, MariaDB) automatically:

```rust
use foundry::prelude::*;

impl SubsystemModule for BillingSubsystem {
    fn slug(&self) -> &'static str { "billing" }
    fn display_name(&self) -> &'static str { "Billing Center" }

    // Code-defined ORM schemas (Prisma-like)
    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            EntitySchema::new("billing_invoices")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("invoice_no", 64).not_null().unique())
                .column(ColumnDef::decimal("amount", 10, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null())
                .column(ColumnDef::json("metadata").default_empty_json())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("updated_at").default_current_timestamp())
                .index(IndexDef::new("idx_invoices_sys", &["system_id"]))
        ]
    }
}
```

CRUD operations are executed via strongly-typed ORM query builders, automatically adapting parameter placeholders (`$1` vs `?`):
```rust
use foundry::prelude::*;

// ORM Insert
let (sql, binds) = OrmInsert::into_table("billing_invoices")
    .value("system_id", "billing")
    .value("invoice_no", "INV-2026-001")
    .value("amount", 99.50)
    .to_sql(Dialect::Postgres);

// ORM Paginated Select
let (sql, binds) = OrmSelect::from_table("billing_invoices")
    .where_eq("system_id", "billing")
    .order_by_desc("created_at")
    .limit(20)
    .offset(0)
    .to_sql(Dialect::Postgres);
```

#### Option B: Zero-DDL Dynamic Models (Instant RESTful Auto-CRUD)
No SQL DDL migrations needed. Define models dynamically via the Admin UI (`/admin`) or manipulate records directly in code:

```rust
use foundry_storage::models::RecordStore;
use serde_json::json;

// Create dynamic record (strictly tenant-isolated by system_slug & model_slug)
let record = RecordStore::create(
    &db,
    &ctx.system_slug,  // subsystem slug
    "articles",         // model slug
    json!({ "title": "First Post", "views": 10 })
).await?;

// Retrieve by ID or query paginated list
let item = RecordStore::get_by_id(&db, &ctx.system_slug, "articles", record.id).await?;
let list = RecordStore::list(&db, &ctx.system_slug, "articles", query).await?;
```
Dynamic models automatically receive REST endpoints at `/api/v1/s/{system_slug}/{model_slug}`.

#### Option C: Native SQL / Script Escape Hatch (Special Triggers, MongoDB Scripts)
For databases without relational ORM support (such as MongoDB) or specialized SQL needs, write custom scripts via `SubsystemModule::migration_ddl` or under `migrations/`.

### 4. Project Validation & Operations

```bash
# Validate project structure and subsystem integrity
foundry validate
```

> **Administrator & IAM Management**: Administrator accounts, roles, and password management are handled visually in the built-in React Admin Dashboard (`/admin/`), keeping the CLI focused purely on development and code lifecycle.

---

## 📚 Example Application

Explore [`examples/blog_platform`](examples/blog_platform) for a complete reference application demonstrating:
- Application bootstrap with `FoundryApp::builder()`
- Multiple custom subsystems (`blog` and `newsletter`)
- Subsystem controllers, business logic services, and validation DTOs
- Custom Admin UI Studio integration
- Custom lifecycle mutation hooks (`BlogMutationHook`)
- Comprehensive integration tests (`tests/integration_test.rs`)

---

## 📖 Documentation

- 🌐 [Official Documentation](https://foundkit.github.io/foundry/)
- 📘 [Getting Started Guide](docs/src/content/docs/getting-started.md)
- 🚀 [Complete System Development Guide](docs/src/content/docs/guides/system-development.md)
- 🏗️ [Architecture Blueprint](docs/src/content/docs/architecture/blueprint.md)
- 🔌 [Subsystems & Extensions Guide](docs/src/content/docs/guides/extensions.md)
- 🗺️ [Development Roadmap](docs/src/content/docs/roadmap.md)

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

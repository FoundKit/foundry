# Foundry

<div align="center">

**A modern, modular, decoupled Rust backend platform and application framework.**

*Build, isolate, and extend independent backend systems and admin control planes with zero upstream coupling.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-16%2B%20%2F%2018%2B-336791.svg)](https://www.postgresql.org)
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
    ├── crates/foundry_storage     # Zero-DDL dynamic models, configs, PostgreSQL, Redis
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
    ├── Cargo.toml                 # [dependencies] foundry = "0.1"
    ├── dev/                       # Local development resources (.gitignore'd)
    │   └── docker-compose.yml     # PostgreSQL 18 + Redis 7 local stack
    ├── src/
    │   ├── main.rs                # Bootstrap with FoundryApp::builder()
    │   └── systems/               # User business subsystems
    │       ├── blog/              # Controllers + Domain Logic + DTOs + Admin Views
    │       └── newsletter/
    ├── config/                    # Application configuration
    └── migrations/                # Application database migrations
```

---

## ⚡ Developer Experience

Creating and running a new Foundry project takes seconds:

```bash
# 1. Install Foundry CLI (from GitHub repository)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# 2. Scaffold a brand new standalone application
foundry new my-app
cd my-app

# 3. Start local development database (PostgreSQL 18 + Redis 7)
docker compose -f dev/docker-compose.yml up -d

# 4. Start the server
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
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
```

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
4. **Zero-DDL Dynamic Storage**: Define system configs and dynamic data models visually in the admin UI without raw database DDL migrations or schema locks.
5. **Instant RESTful Auto-CRUD**: Dynamic data models automatically receive REST endpoints (`/api/v1/s/{system_slug}/{model_slug}`) with pagination and GIN JSON search.
6. **Decoupled Admin Shell & SDK Bridge**: The Admin SPA (`apps/admin`) embeds subsystem custom pages (HTML, React, Vue) inside sandboxed views, injecting JWT authentication and theme settings automatically via `postMessage` protocol.
7. **Hierarchical RBAC**: Fine-grained role-based access control with Super Admins (`super_admin`), Platform Admins (`admin`), and Topic-Scoped Admins (`topic_admin`).
8. **Lifecycle Mutation Hooks**: Intercept entity lifecycle events (`before_create`, `after_create`, `before_update`, `after_update`, `before_delete`, `after_delete`) using `MutationHook`.

---

## 📦 Workspace Crates

| Crate | Description |
|---|---|
| [`foundry`](crates/foundry) | Top-level facade crate providing `FoundryApp`, `FoundryBuilder`, and `prelude`. |
| [`foundry_core`](crates/foundry_core) | Core primitives, `SystemContext`, `SubsystemModule`, `AppError`, and response models. |
| [`foundry_storage`](crates/foundry_storage) | PostgreSQL connection pool, Redis cache, Zero-DDL dynamic models, and migrations. |
| [`foundry_auth`](crates/foundry_auth) | Admin identities, Argon2id hashing, JWT token service, and Topic RBAC guards. |
| [`foundry_engine`](crates/foundry_engine) | Unified Axum routing, Auto-CRUD handler engine, and audit logging middleware. |
| [`foundry_extension`](crates/foundry_extension) | Lifecycle mutation hook pipelines and extension mechanisms. |
| [`foundry_cli`](crates/foundry_cli) | Developer CLI binaries (`foundry` & `foundry-cli`) for project and subsystem scaffolding. |

---

## 🛠️ CLI Commands & Core Workflows

Foundry provides a streamlined developer CLI (`foundry` / `foundry-cli`) to manage applications, subsystems, and models.

### 1. Local Development Database

Scaffolded projects include a dedicated `dev/docker-compose.yml` (ignored by `.gitignore` to keep repositories clean):

```bash
# Start local PostgreSQL 18 and Redis 7
docker compose -f dev/docker-compose.yml up -d

# Check service status
docker compose -f dev/docker-compose.yml ps

# Stop local database
docker compose -f dev/docker-compose.yml down
```

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

# 4. Create an external standalone subsystem (with subsystem.json & custom pages)
foundry system new-external carnival --name "Carnival 2026"

# 5. List all discovered subsystems
foundry system list
```

### 3. Data Model Creation & Storage

Choose between Zero-DDL dynamic models and strongly-typed SQLx schemas:

#### Option A: Zero-DDL Dynamic Models (Instant RESTful Auto-CRUD)
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
let list = RecordStore::list(&db, &ctx.system_slug, "articles", 1, 20).await?;
```
Dynamic models automatically receive REST endpoints at `/api/v1/s/{system_slug}/{model_slug}`.

#### Option B: Native SQL Migrations & Typed Models (High-concurrency & Transactions)
1. Add migration in `migrations/001_create_orders.sql`:
   ```sql
   CREATE TABLE IF NOT EXISTS orders (
       id BIGSERIAL PRIMARY KEY,
       system_slug VARCHAR(64) NOT NULL,
       order_no VARCHAR(64) NOT NULL UNIQUE,
       amount BIGINT NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   CREATE INDEX IF NOT EXISTS idx_orders_system_slug ON orders(system_slug);
   ```
2. Apply migration:
   ```bash
   foundry migrate
   # Or set AUTO_MIGRATE=true in .env to auto-apply on startup
   ```
3. Query via SQLx with `#[derive(sqlx::FromRow)]`.

### 4. Admin IAM & Project Validation

```bash
# Validate project structure and manifest integrity
foundry validate

# Create an administrator account
foundry admin create --username admin --password secret --role super_admin

# Reset administrator password
foundry admin reset-password --username admin --new-password newsecret
```

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
- 🏗️ [Architecture Blueprint](docs/src/content/docs/architecture/blueprint.md)
- 🔌 [Subsystems & Extensions Guide](docs/src/content/docs/guides/extensions.md)
- 🗺️ [Development Roadmap](docs/src/content/docs/roadmap.md)

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

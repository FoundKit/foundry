# Foundry

<div align="center">

**A modern, modular, decoupled Rust backend platform and application framework.**

*Build, isolate, and extend independent backend systems and admin control planes with zero upstream coupling.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-16%2B%20%2F%2017%2B-336791.svg)](https://www.postgresql.org)
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

# 3. Enter your project directory and start the server
cd my-app
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

## 🛠️ CLI Commands

```bash
# Create a new standalone application
foundry new <project-name>

# Scaffold a new subsystem in an existing project
foundry system new <slug> --name "Display Name"

# Apply baseline database migrations
foundry migrate --database-url postgres://...

# Create an administrator account
foundry admin create --username admin --password secret --role super_admin

# Reset administrator password
foundry admin reset-password --username admin --new-password newsecret

# Validate application structure and manifests
foundry validate
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

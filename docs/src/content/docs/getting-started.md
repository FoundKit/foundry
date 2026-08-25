---
title: Getting Started
description: Quick start guide to creating, building, and running applications with the Foundry Framework.
---

# Getting Started with Foundry

Foundry is a modern, modular Rust backend platform and framework that empowers developers to build scalable multi-system backends with embedded admin control planes.

---

## ⚡ 5-Minute Quick Start

### 1. Install the Foundry CLI

Install the `foundry` CLI binary using Cargo:

```bash
cargo install foundry-cli
```

Verify your installation:

```bash
foundry --help
```

---

### 2. Scaffold a New Project

Create a brand new standalone user application:

```bash
foundry new my-app
cd my-app
```

This generates a clean, self-contained project:

```text
my-app/
├── Cargo.toml                # Depends on `foundry = "0.1"`
├── src/
│   ├── main.rs               # Application bootstrap
│   └── systems/
│       ├── mod.rs
│       └── sample/           # Default starter subsystem
│           ├── controllers/  # Axum routes (/api/v1/s/sample/ext/*)
│           ├── logic/        # Domain business logic
│           ├── dto/          # Validation schemas
│           ├── custom_pages/ # Custom Admin UI
│           └── mod.rs
├── migrations/               # User database migrations
├── .env                      # Local configuration
└── README.md
```

---

### 3. Start Database Services

Start local PostgreSQL and Redis instances:

```bash
docker run -d --name foundry-postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry -p 5432:5432 postgres:18-alpine
docker run -d --name foundry-redis -p 6379:6379 redis:8-alpine
```

---

### 4. Run the Application

```bash
cargo run
```

Foundry will automatically connect to PostgreSQL, run baseline schema migrations, mount all registered subsystems, and start listening on `http://127.0.0.1:8080`.

---

## 🏗️ How Applications Consume Foundry

In your application's `Cargo.toml`:

```toml
[dependencies]
foundry = "0.1"
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
```

In `src/main.rs`:

```rust
pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config from .env or environment variables
    let config = FoundryConfig::from_env();

    // 2. Build the application with subsystems and hooks
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(SampleSubsystem)
        .build()
        .await?;

    // 3. Serve traffic
    app.run().await?;
    Ok(())
}
```

---

## 🧩 Adding a Custom Subsystem

Scaffold a new business subsystem inside your project:

```bash
foundry system new blog --name "Blog & Publishing"
```

Register it in `src/main.rs`:

```rust
use systems::BlogSubsystem;

let app = FoundryApp::builder()
    .register_subsystem(BlogSubsystem)
    // ...
    .build()
    .await?;
```

Your new subsystem endpoints will be immediately accessible under `/api/v1/s/blog/ext/*`.

---

## 📦 Next Steps

- Explore the [Architecture Blueprint](../architecture/blueprint/) to understand how Framework and Application code are isolated.
- Read the [Subsystems & Extensions Guide](../guides/extensions/) for advanced controllers, services, and Admin UI pages.
- Check out the [Roadmap & Versioning](../roadmap/) for upgrade policies and release lifecycles.

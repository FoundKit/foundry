---
title: Getting Started
description: Quick start guide to creating, building, and running standalone applications using the Foundry Framework via Git.
---

# Getting Started with Foundry

Foundry is a modern, modular Rust backend platform and framework. During the current development and testing phase (prior to official publication on crates.io), developers consume Foundry as a **Git Dependency** in their own independent repositories.

---

## ⚡ 5-Minute Quick Start

### 1. Install the Foundry CLI

Install the `foundry` CLI directly from the GitHub repository using Cargo:

```bash
# Install from GitHub repository
cargo install --git https://github.com/foundkit/foundry foundry_cli
```

Verify your installation:

```bash
foundry --help
```

> **Tip (Local Development)**: If you have cloned the Foundry source repository locally, compile and install directly from the local path:
> ```bash
> cargo install --path crates/foundry_cli
> ```

---

### 2. Scaffold a New Application

Create a brand new standalone user application:

```bash
foundry new my-app
cd my-app
```

The CLI generates a clean, self-contained Rust project with a pre-configured Git dependency:

```text
my-app/
├── Cargo.toml                # Pre-configured with: foundry = { git = "...", branch = "main" }
├── dev/                      # Dedicated local dev resources (.gitignore'd)
│   └── docker-compose.yml    # Local PostgreSQL 18 & Redis 7 stack
├── src/
│   ├── main.rs               # Application bootstrap with FoundryApp::builder()
│   └── systems/
│       ├── mod.rs
│       └── sample/           # Starter business subsystem
│           ├── controllers/  # Axum routes (/api/v1/s/sample/ext/*)
│           ├── logic/        # Domain business logic
│           ├── dto/          # Request validation schemas
│           ├── custom_pages/ # Custom Admin UI Studio
│           └── mod.rs
├── migrations/               # User database migrations
├── .env                      # Local environment configuration
└── README.md
```

#### Inside `Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }
tokio = { version = "1.44", features = ["full"] }
axum = { version = "0.8", features = ["macros"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
validator = { version = "0.20", features = ["derive"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
```

---

### 3. Start Database Services

Start local PostgreSQL and Redis containers using the pre-configured `dev/docker-compose.yml` (the `dev/` directory is ignored by `.gitignore` to keep user repositories clean):

```bash
docker compose -f dev/docker-compose.yml up -d
```

Check your `.env` file matches your local setup:

```bash
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
AUTO_MIGRATE=true
```

---

### 4. Run the Application

Start the backend server:

```bash
cargo run
```

When the server starts, Foundry will automatically:
1. Connect to PostgreSQL and Redis.
2. Apply database migrations (when `AUTO_MIGRATE=true`).
3. Mount all registered subsystems.
4. Start listening on `http://0.0.0.0:8080` (accessible via `http://localhost:8080`).

---

### 5. Access the Admin Control Plane

Open your browser and navigate to:

```text
http://localhost:8080/admin
```

#### Default Administrator Credentials:
* **Username**: `admin`
* **Password**: `admin123456`
* **Role**: `super_admin`

#### Creating Custom Administrators via CLI:
You can create new administrator accounts anytime using the CLI:

```bash
foundry admin create --username developer --password devsecret --role super_admin
```

---

### 6. Verify APIs

#### Health Check:
```bash
curl http://localhost:8080/api/v1/health
# Response: OK
```

#### Subsystem Extension Endpoint:
```bash
curl -X POST http://localhost:8080/api/v1/s/sample/ext/greet \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'

# Response:
# {
#   "code": 0,
#   "message": "success",
#   "data": {
#     "message": "Hello, Alice! Welcome to Foundry Framework."
#   }
# }
```

---

## 🧩 Next Steps

- **[Database & Custom Storage Guide](../guides/database/)**: Learn how to write custom SQL queries, transactions, dynamic models, and migrations.
- **[Subsystems & Custom Features](../guides/extensions/)**: Learn the 3-Layer pattern and how to build Admin UI extensions.
- **[CLI Reference Guide](../guides/cli/)**: Master all CLI scaffolding and admin management commands.

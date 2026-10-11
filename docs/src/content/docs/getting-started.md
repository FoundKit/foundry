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
├── migrations/               # (Optional) On-demand generated via `foundry ddl` (init.sql / init.js)
├── .env                      # Local environment configuration (with AUTO_MIGRATE switch)
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

### 3. Start & Initialize Database Services

Foundry connects to your database via `DATABASE_URL` in `.env`.

#### Option 1: Run Local Containers (Docker)
You can start your preferred database and Redis instance using standard Docker commands:

```bash
# PostgreSQL 18 & Redis 7
docker run -d --name my-postgres -p 5432:5432 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry postgres:18-alpine
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# Or for MySQL 8.4:
# docker run -d --name my-mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry mysql:8.4
```

#### Option 2: Connect Existing Databases
If connecting to an external database, simply update `DATABASE_URL` and `DATABASE_TYPE` in `.env`. When `AUTO_MIGRATE=true`, Foundry will automatically run schema checks, initialize tables/indexes, and seed the superadmin user on startup (`cargo run`).

Configure your `.env` for your preferred database:

```bash
HOST=0.0.0.0
PORT=8080

# Option A: PostgreSQL (default)
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry

# Option B: MySQL / MariaDB (requires features = ["mysql"] in Cargo.toml)
# DATABASE_URL=mysql://root:root@localhost:3306/foundry

REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
# Auto-check and migrate framework tables on startup
AUTO_MIGRATE=true
```

---

### 4. Run the Application

Start the backend server:

```bash
cargo run
```

When the server starts, Foundry will automatically:
1. Connect to the configured database (PostgreSQL / MySQL, etc.) and Redis.
2. Verify metadata schema (when `AUTO_MIGRATE=true`).
3. Serve the embedded React Admin SPA and mount all registered subsystems.
4. Start listening on `http://0.0.0.0:8080`.

---

### 5. Access the Admin Control Plane

Open your browser and navigate to the root or admin URL (root automatically redirects to the admin portal):

```text
http://localhost:8080/
# or
http://localhost:8080/admin/
```

#### Default Administrator Credentials:
* **Username**: `admin`
* **Password**: `admin123456`
* **Role**: `super_admin`

#### Administrator & IAM Management:
A default super administrator is pre-seeded on startup. To create additional administrators, reset passwords, or configure subsystem permissions, manage them visually in the Admin Web Console under the "Administrators" menu.

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

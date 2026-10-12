---
title: Getting Started
description: Quick start guide to creating, building, and running standalone applications using the Foundry Framework with PostgreSQL, MySQL, MariaDB, or MongoDB.
---

# Getting Started with Foundry

Foundry is a modern, modular, fully decoupled Rust backend platform and framework. During the current development and testing phase (prior to official publication on crates.io), developers consume Foundry as a **Git Dependency** in their own independent repositories.

> [!NOTE]
> **Enterprise Multi-Database SPI Support**: Foundry natively supports four major database engines through a unified Zero-Driver-Leakage SPI:
> 1. **PostgreSQL (14+)** *(Default)* — Relational engine with native JSONB and GIN inverted indexing.
> 2. **MongoDB (6.0+)** — Native BSON document engine with atomic sequence counters and dynamic wildcard indexing (Zero-DDL).
> 3. **MySQL (8.0+)** — Universal single-table relational engine with covering indexes and deferred join pagination.
> 4. **MariaDB (10.5+)** — Dedicated MariaDB dialect engine with fine-tuned InnoDB indexes.

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

> [!TIP]
> **Local Development**: If you have cloned the Foundry source repository locally, compile and install directly from the local path:
> ```bash
> cargo install --path crates/foundry_cli
> ```

---

### 2. Scaffold a New Application

Use `foundry new` to create a standalone user application. You can select your preferred database engine using the `--db` (or `-d`) flag:

```bash
# Option 1: Default (PostgreSQL)
foundry new my-app

# Option 2: MongoDB (Zero-DDL Document Engine)
foundry new my-app --db mongodb

# Option 3: MySQL (8.0+)
foundry new my-app --db mysql

# Option 4: MariaDB (10.5+)
foundry new my-app --db mariadb

cd my-app
```

The CLI generates a clean, self-contained Rust project with matching Cargo feature flags and environment configuration:

```text
my-app/
├── Cargo.toml                # Pre-configured with database feature flags & Foundry dependency
├── src/
│   ├── main.rs               # Application bootstrap with FoundryApp::builder()
│   └── systems/
│       ├── mod.rs
│       └── sample/           # Starter business subsystem
│           ├── controllers/  # Axum routes (/api/v1/s/sample/ext/*)
│           ├── logic/        # Domain business logic
│           ├── dto/          # Request validation schemas
│           ├── custom_pages/ # Custom Admin UI Studio (HTML / React iframe)
│           └── mod.rs
├── migrations/               # (Optional) On-demand dialect scripts via `foundry ddl` (init.sql / init.js)
├── .env                      # Local environment configuration (DATABASE_URL, DATABASE_TYPE, AUTO_MIGRATE)
├── .env.example              # Environment variables template
└── README.md
```

#### Inside `Cargo.toml`:

Depending on the database chosen during scaffolding, `Cargo.toml` is automatically configured:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
# --- Database Selection in Foundry ---
# Option A: PostgreSQL (Default)
foundry = { git = "https://github.com/foundkit/foundry", branch = "main" }

# Option B: MongoDB (Zero-DDL Document Engine)
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mongodb"] }

# Option C: MySQL
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mysql"] }

# Option D: MariaDB
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", default-features = false, features = ["standard", "mariadb"] }

# Option E: Multi-Database Support (Enables all engines simultaneously)
# foundry = { git = "https://github.com/foundkit/foundry", branch = "main", features = ["all-databases"] }

# --- Runtime & Web Dependencies ---
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

### 3. Start & Initialize Database and Cache Services

Foundry connects to your database via `DATABASE_URL` and `DATABASE_TYPE` in `.env`, and utilizes Redis for caching and distributed locks.

#### Option 1: Run Local Containers (Docker)

Start your preferred database and Redis 7 container using Docker:

```bash
# 1. Start Redis 7 (Required by all storage backends)
docker run -d --name my-redis -p 6379:6379 redis:7-alpine

# --- Choose ONE of the following database engines ---

# Option A: PostgreSQL 18 (Default)
docker run -d --name my-postgres -p 5432:5432 \
  -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry \
  postgres:18-alpine

# Option B: MongoDB 7 (Zero-DDL Document Engine)
docker run -d --name my-mongodb -p 27017:27017 \
  mongo:7

# Option C: MySQL 8.4
docker run -d --name my-mysql -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry \
  mysql:8.4

# Option D: MariaDB 11
docker run -d --name my-mariadb -p 3307:3306 \
  -e MARIADB_ROOT_PASSWORD=root -e MARIADB_DATABASE=foundry \
  mariadb:11
```

#### Option 2: Configure Environment Variables (`.env`)

Configure your `.env` file according to your selected database:

```bash
HOST=0.0.0.0
PORT=8080

# -----------------------------------------------------------------------------
# Database Configuration (Choose Option A, B, C, or D)
# -----------------------------------------------------------------------------

# Option A: PostgreSQL (Default)
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
DATABASE_TYPE=postgres

# Option B: MongoDB (Zero-DDL, requires features = ["standard", "mongodb"] in Cargo.toml)
# DATABASE_URL=mongodb://127.0.0.1:27017/foundry
# DATABASE_TYPE=mongodb

# Option C: MySQL (Requires features = ["standard", "mysql"] in Cargo.toml)
# DATABASE_URL=mysql://root:root@localhost:3306/foundry
# DATABASE_TYPE=mysql

# Option D: MariaDB (Requires features = ["standard", "mariadb"] in Cargo.toml)
# DATABASE_URL=mariadb://root:root@localhost:3307/foundry
# DATABASE_TYPE=mariadb

# -----------------------------------------------------------------------------
# Cache & Security Configuration
# -----------------------------------------------------------------------------
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production

# Automatic database schema check and superadmin seeding on startup
AUTO_MIGRATE=true
```

> [!IMPORTANT]
> **Understanding Automatic Migration vs. Strict DBA Mode**:
> - **When `AUTO_MIGRATE=true` (Default)**:
>   Foundry checks and initializes the database on startup (`cargo run`):
>   - **For SQL Engines (PostgreSQL / MySQL / MariaDB)**: Verifies table structures, creates missing platform tables, applies covering indexes, and seeds the superadmin user.
>   - **For MongoDB**: Verifies collections (`records`, `models`, `systems`, `configs`, `admins`, `audit_logs`), creates unique and wildcard indexes, and seeds the default superadmin document without any manual DDL.
> - **When `AUTO_MIGRATE=false` (Strict DBA Production Auditing)**:
>   You can generate offline initialization scripts using the CLI:
>   - SQL: `foundry ddl -d postgres` (or `mysql` / `mariadb`) generates `migrations/init.sql`.
>   - MongoDB: `foundry ddl -d mongodb` generates `migrations/init.js` and `migrations/README.md`. DBAs can execute it directly via:
>     ```bash
>     mongosh "mongodb://127.0.0.1:27017/foundry" migrations/init.js
>     ```

---

### 4. Run the Application

Start the backend server:

```bash
cargo run
```

When the server starts, Foundry will automatically:
1. Connect to the configured database (**PostgreSQL**, **MongoDB**, **MySQL**, or **MariaDB**) using smart protocol sniffing.
2. Initialize connections to Redis.
3. Verify metadata tables/collections and ensure the superadmin user exists (`AUTO_MIGRATE=true`).
4. Mount the embedded React Admin SPA and register all user subsystems.
5. Begin listening on `http://0.0.0.0:8080`.

---

### 5. Access the Admin Control Plane

Open your browser and navigate to the application URL (visiting `/` automatically redirects to the admin portal):

```text
http://localhost:8080/
# or
http://localhost:8080/admin/
```

#### Default Administrator Credentials:
* **Username**: `admin`
* **Password**: `admin123456`
* **Role**: `super_admin`

#### Admin Capabilities & Governance:
* **Administrators & IAM**: Manage team members, reset passwords, and bind fine-grained subsystem permissions.
* **Zero-DDL Data Studio**: Visually inspect dynamic models, fields, and records across PostgreSQL, MySQL, MariaDB, or MongoDB.
* **Subsystem Console**: Embedded UI extensions (like the starter sample subsystem dashboard) are rendered directly inside the Admin Shell.
* **Audit Logs & Configs**: Track platform activity logs and manage tenant-scoped system configurations.

---

### 6. Verify APIs

#### 1. System Health Check:
```bash
curl http://localhost:8080/api/v1/health
# Response: OK
```

#### 2. Administrator Login & JWT Authentication:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123456"}'

# Response:
# {
#   "code": 0,
#   "message": "success",
#   "data": {
#     "token": "eyJhbGciOiJIUzI1NiIsIn...",
#     "user": {
#       "username": "admin",
#       "role": "super_admin"
#     }
#   }
# }
```

#### 3. Subsystem Extension Endpoint:
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

### 7. Essential CLI Developer Commands

The Foundry CLI includes rich tooling for scaffolded projects:

```bash
# Scaffold a new business subsystem (e.g., billing, orders, cms)
foundry system new billing --name "Billing Center"

# List all discovered subsystems in current project
foundry system list

# Export schema migration or MongoDB initialization script for DBA review
foundry ddl --db postgres    # Outputs migrations/init.sql
foundry ddl --db mongodb     # Outputs migrations/init.js

# Validate project structure, manifests, and subsystem conventions
foundry validate
```

---

## 🧩 Next Steps

- **[System Development Complete Guide](../guides/system-development/)**: Master the ThinkPHP/NestJS-style 3-Layer architecture, chainable ORM queries, and end-to-end development.
- **[Database & Custom Storage Guide](../guides/database/)**: In-depth guide to Multi-Database SPI, MongoDB BSON storage, PostgreSQL GIN indexing, and custom ORM schemas.
- **[Subsystems & Custom Features](../guides/extensions/)**: Build business domain controllers, lifecycle hooks, and custom Admin UI pages.
- **[CLI Tooling Reference](../guides/cli/)**: Master all CLI flags, scaffolding options, and maintenance commands.
- **[Storage Engines Architecture](../architecture/storage-engines/)**: Architectural blueprints and multi-database roadmaps.

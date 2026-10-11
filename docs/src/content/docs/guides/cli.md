---
title: CLI Tooling Reference
description: Complete command-line interface reference for foundry and foundry-cli developer tooling.
---

# Foundry CLI Tooling Reference

Foundry provides a comprehensive command-line tool available as both `foundry` and `foundry-cli`.

---

## 📦 Installation

```bash
# Option 1: Install directly from GitHub repository (Pre-release)
cargo install --git https://github.com/foundkit/foundry foundry_cli

# Option 2: Install from local cloned repository
cargo install --path crates/foundry_cli
```

---

## 🛠️ Commands Overview

| Command | Description |
|---|---|
| `foundry new <name> [--db <type>]` | Scaffold a brand new standalone application project |
| `foundry system new <slug>` | Scaffold a new business subsystem code skeleton within current application |
| `foundry system list` | List all discovered subsystems in current project |
| `foundry migrate` | Apply baseline schema and project migrations |
| `foundry ddl --db <type>` | Generate dialect-specific DDL / migration scripts |
| `foundry validate` | Validate application structure, subsystems, and standards |

---

## 1. Project Scaffolding: `foundry new`

Creates a complete standalone Rust backend application.

```bash
# Standard project creation (defaults to PostgreSQL and Git dependency on main branch)
foundry new my-app

# Scaffold with a specific database engine (postgres, mysql, mariadb, mongodb)
foundry new my-app --db mysql
foundry new my-app --db mariadb
foundry new my-app --db mongodb

# Create with specific local path (for monorepos or local framework development)
foundry new my-app --path /path/to/foundry/crates/foundry

# Create pointing to specific git branch or tag
foundry new my-app --git https://github.com/foundkit/foundry --branch feature/next

# Create pinned to a specific crates.io version
foundry new my-app --version 0.1.0
```

> **Database Configuration & Clean Skeleton**: Projects generated with `foundry new --db <type>` create a clean project skeleton (without creating a `migrations/` directory by default), setting `DATABASE_URL` and `DATABASE_TYPE` in `.env`. When `AUTO_MIGRATE=true`, running `cargo run` automatically uses compile-time embedded DDL to initialize schemas, indexes, and the default superadmin without needing physical SQL files. When DBA auditing or external tools are used, run `foundry ddl` to export dialect migration scripts.

---

## 2. Subsystem Management: `foundry system`

### Scaffolding a Subsystem: `foundry system new`

Scaffolds a new self-contained business domain directory inside `src/systems/<slug>/`:

```bash
foundry system new billing --name "Billing & Payments"
```

Generates:
* `src/systems/billing/mod.rs` (implements `SubsystemModule`)
* `src/systems/billing/controllers/mod.rs` (Axum routes)
* `src/systems/billing/logic/mod.rs` (Domain business logic)
* `src/systems/billing/dto/mod.rs` (Validation structs)
* `src/systems/billing/custom_pages/` (Custom Admin UI views)

### Listing Subsystems: `foundry system list`

Scans and lists all registered subsystems:

```bash
foundry system list
```

---

## 3. Database Migrations & DDL Generation

### Running Database Migrations: `foundry migrate`

Applies migrations against the target database:

```bash
# Migrate current project (auto-detects DATABASE_URL and DATABASE_TYPE from .env)
foundry migrate

# Explicit connection string and database type
foundry migrate --database-url postgres://postgres:postgrespassword@localhost:5432/foundry --database postgres

# Generate dialect DDL into migrations/ without connecting to database
foundry migrate --generate
```

> 💡 **Architecture Note**: `foundry migrate` (and startup `AUTO_MIGRATE=true`) executes **compile-time embedded dialect DDL / native driver logic** inside the binary, which does not require a physical `migrations/` directory on disk.

### Generating Dialect DDL: `foundry ddl`

`foundry ddl` reads baseline dialect schemas directly from compiled binary storage driver providers (`foundry_storage`), automatically synthesizing any code-defined DDL extensions registered by subsystems or plugins.

Exports dialect-specific schema initialization scripts for DBA review or external tooling (creates `migrations/` directory):

```bash
# Generate for current project (reads DATABASE_TYPE from .env)
foundry ddl

# Explicit engine selection
foundry ddl --db mysql        # Generates migrations/init.sql
foundry ddl --db postgres     # Generates migrations/init.sql
foundry ddl --db mariadb      # Generates migrations/init.sql
foundry ddl --db mongodb      # Generates migrations/init.js and migrations/README.md

# Output to stdout or custom path
foundry ddl --db mysql -o -
foundry ddl --db postgres -o deploy/init.sql
```

#### Code-First ORM Architecture & Schema Compilation

Foundry adopts an **ORM-First (Prisma-like)** architecture. For databases supporting relational ORM (PostgreSQL, MySQL, MariaDB), both `foundry migrate` and `foundry ddl` compile directly from code-defined ORM schemas into dialect DDL:

1. **Subsystem In-Code ORM Schema Definition (Prisma-like, Recommended)**:
   Implement `orm_models` on `SubsystemModule` to declare strongly-typed entity schemas:
   ```rust
   impl SubsystemModule for BillingSubsystem {
       fn slug(&self) -> &'static str { "billing" }
       fn display_name(&self) -> &'static str { "Billing Center" }
       fn register_routes(&self, router: Router) -> Router { router }

       // Declare ORM models in code (Prisma-like)
       fn orm_models(&self) -> Vec<EntitySchema> {
           vec![
               EntitySchema::new("billing_invoices")
                   .column(ColumnDef::bigint("id").primary_key().auto_increment())
                   .column(ColumnDef::varchar("system_id", 32).not_null())
                   .column(ColumnDef::varchar("invoice_no", 64).not_null().unique())
                   .column(ColumnDef::decimal("amount", 10, 2).not_null())
                   .column(ColumnDef::varchar("status", 24).not_null())
                   .column(ColumnDef::datetime("created_at").default_current_timestamp())
                   .index(IndexDef::new("idx_invoices_sys", &["system_id"]))
           ]
       }
   }
   ```
   When registered via `app.register_subsystem(BillingSubsystem)`, the framework collects these ORM models. Running `foundry ddl --db postgres` (or `mysql` / `mariadb`) or running migrations will automatically compile and synchronize your subsystem tables alongside core platform tables!

2. **Encapsulated ORM CRUD Query Builder**:
   Business services perform database operations using the type-safe ORM query builder without manually writing SQL strings:
   ```rust
   // ORM Insert
   let (sql, binds) = OrmInsert::into_table("billing_invoices")
       .value("system_id", "billing")
       .value("invoice_no", "INV-001")
       .value("amount", 99.0)
       .to_sql(Dialect::Postgres);

   // ORM Select
   let (sql, binds) = OrmSelect::from_table("billing_invoices")
       .where_eq("system_id", "billing")
       .limit(10)
       .to_sql(Dialect::Postgres);
   ```

3. **Raw SQL & Script Escape Hatch (Fallback)**:
   For databases without relational ORM (e.g. MongoDB) or specialized custom SQL (triggers, extensions), use `migration_ddl`:
   ```rust
   fn migration_ddl(&self, db_type: &str) -> Option<String> {
       match db_type {
           "postgres" => Some("CREATE INDEX IF NOT EXISTS idx_custom ON ...;".into()),
           _ => None,
       }
   }
   ```

4. **Custom Database Storage Provider (StorageDriverProvider)**:
   To support new database engines (such as SQLite or CockroachDB), implement `StorageDriverProvider` (implementing `get_baseline_ddl()`, `create_engines()`, and `run_migrations()`) and inject it into the runtime via `foundry_storage::register_storage_provider(Arc::new(MyCustomProvider))`.

---

## 4. Validation: `foundry validate`

Verifies that the current project contains valid subsystem modules and structure:

```bash
foundry validate
```

---

## 5. Administrator & IAM Operations

Following best practices for separation of concerns:
* **CLI focuses on developer tooling**: Project scaffolding, subsystem generation, migration execution, and structure validation.
* **Web Admin UI manages runtime IAM**: Administrator account creation, role assignment (`super_admin` / `admin` / `topic_admin`), and password resetting are handled directly in the built-in React Admin Dashboard (`/admin/`).
* Projects initialize with a default superadmin account (`admin / admin123456`) upon startup. Operators manage accounts securely through the Web UI without passing plain-text credentials over CLI terminals.

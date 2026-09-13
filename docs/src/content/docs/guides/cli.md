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
| `foundry new <name>` | Scaffold a brand new standalone application project |
| `foundry system new <slug>` | Scaffold a new business subsystem code skeleton within current application |
| `foundry system list` | List all discovered subsystems in current project |
| `foundry migrate` | Apply baseline schema and project SQL migrations |
| `foundry validate` | Validate application structure, subsystems, and standards |

---

## 1. Project Scaffolding: `foundry new`

Creates a complete standalone Rust backend application.

```bash
# Standard project creation (defaults to Git dependency on main branch)
foundry new my-app

# Create with specific local path (for monorepos or local framework development)
foundry new my-app --path /path/to/foundry/crates/foundry

# Create pointing to specific git branch or tag
foundry new my-app --git https://github.com/foundkit/foundry --branch feature/next

# Create pinned to a specific crates.io version
foundry new my-app --version 0.1.0
```

> **Dedicated Dev Resource Directory**: Scaffolded projects include `dev/docker-compose.yml` (ignored by `.gitignore` to keep repositories clean), enabling one-command local startup of PostgreSQL 18 and Redis 7:
> ```bash
> cd my-app
> docker compose -f dev/docker-compose.yml up -d
> ```

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

## 3. Database Migrations: `foundry migrate`

Applies SQL migrations against PostgreSQL:

```bash
foundry migrate --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

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

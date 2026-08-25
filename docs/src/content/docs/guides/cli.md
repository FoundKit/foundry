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
| `foundry system new <slug>` | Scaffold a new business subsystem within the current application |
| `foundry system new-external <slug>` | Scaffold a standalone external file-based subsystem directory |
| `foundry system list` | List all discovered code-first and external subsystems |
| `foundry migrate` | Apply baseline schema and application database migrations |
| `foundry admin create` | Create a new administrator account |
| `foundry admin reset-password` | Reset the password of an existing administrator |
| `foundry validate` | Validate application structure, subsystems, and manifests |

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

---

## 2. Subsystem Management: `foundry system`

### Code-First Subsystem: `foundry system new`

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

### External Standalone Subsystem: `foundry system new-external`

Scaffolds a standalone file-based subsystem with a `subsystem.json` manifest:

```bash
foundry system new-external carnival_2026 --name "Carnival 2026"
```

### List Discovered Subsystems: `foundry system list`

Scans and lists all code-first and external subsystems:

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

## 4. Administrator IAM: `foundry admin`

### Creating an Administrator:
```bash
foundry admin create \
  --username admin \
  --password mysecretpassword \
  --role super_admin \
  --allowed "*" \
  --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

#### Roles Available:
* `super_admin`: Full universal privileges across all subsystems and administrator management.
* `admin`: Platform-level administrator privileges across all subsystems.
* `topic_admin`: Scoped administrator restricted only to assigned subsystems (specified via `--allowed "slug1,slug2"`).

### Resetting an Administrator's Password:
```bash
foundry admin reset-password \
  --username admin \
  --new-password newsecretpassword \
  --database-url postgres://postgres:postgrespassword@localhost:5432/foundry
```

---

## 5. Validation: `foundry validate`

Verifies that the current project contains valid subsystem modules and structure:

```bash
foundry validate
```

# Foundry

<div align="center">

**Build complete multi-tenant systems from a shared foundation.**

*Foundry is an open-source platform for building, isolating, and extending independent backend systems and admin panels.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-18.6%2B-336791.svg)](https://www.postgresql.org)
[![Redis](https://img.shields.io/badge/redis-8%2B-dc382d.svg)](https://redis.io)

</div>

---

## 🌟 Key Capabilities

1. **Base Platform & Subsystems Complete Decoupling**: Complete physical and architectural isolation between the base infrastructure platform and custom business subsystems. Subsystem code can be hosted in an independent Git repository with zero upstream merge conflicts.
2. **Self-Contained Subsystem Standard**: Each subsystem folder encapsulates its own API controllers, domain logic services, DTO validation schemas, custom admin UI pages (`custom_pages/`), and manifest (`subsystem.json`).
3. **Zero-DDL Dynamic Storage Engine**: Visually define system configs and dynamic data models without database DDL migrations or metadata locks.
4. **Automatic RESTful CRUD**: Instant REST APIs generated for all dynamic models (`/api/v1/s/{system_slug}/{model_slug}`).
5. **Decoupled RESTful Route Architecture**: Standardized REST paths separating Admin API (`/api/v1/admin/*`), Auto-CRUD (`/api/v1/s/{slug}/*`), and Custom Extension APIs (`/api/v1/s/{slug}/ext/*`).
6. **Subsystem Custom Admin UI Pages & SDK Bridge**: Register custom admin pages seamlessly integrated into the Foundry Admin UI shell with automatic JWT token and theme injection via `FoundryBridge`.
7. **Hierarchical Admin IAM & Topic-Scoped RBAC**: Super Admin (`super_admin`) and scoped Topic Admins (`allowed_systems: ["carnival_demo"]`).
8. **Unified Release Packaging Pipeline**: One-command release build script (`./scripts/build-release.sh`) and multi-stage Docker build merging base infra and custom subsystems into a single deployable artifact.

---

## 📁 Repository Structure

```
foundry/
├── apps/
│   ├── server/           # Foundry Core Server (Axum binary)
│   ├── admin/            # Visual Admin Dashboard SPA (React + TypeScript + Tailwind)
│   └── cli/              # Developer CLI Tool (Subsystem scaffolding & repo validation)
├── crates/
│   ├── foundry_core/     # SystemContext, SubsystemModule, CustomAdminPageSpec
│   ├── foundry_storage/  # PostgreSQL Zero-DDL storage & dynamic model ORM
│   ├── foundry_auth/     # Admin IAM, Argon2id, JWT & Topic RBAC
│   ├── foundry_engine/   # Multi-system router, Auto-CRUD, Audit middleware
│   └── foundry_extension/# Mutation hooks & WASM extension pipeline
├── systems/              # Sub-System workspace (Git Submodule mount point for custom repo)
│   └── src/<slug>/       # Self-contained subsystem (API + Logic + DTO + Custom Admin UI)
├── external_systems/     # Standalone external subsystem repositories
├── scripts/              # Build & Packaging Tooling (build-release.sh)
├── migrations/           # Baseline database initialization (`init.sql`)
└── docker/               # Docker container configurations
```

---

## 🚀 Quick Scaffolding & Packaging Commands

### 1. Initialize a Standalone Custom Subsystems Git Repository

```bash
cargo run --bin foundry-cli -- system init-repo ../my-foundry-systems
```

### 2. Scaffold a Self-Contained Subsystem (Compiled Monorepo)

```bash
cargo run --bin foundry-cli -- system new carnival_demo --name "Carnival Demo"
```

### 3. Validate Subsystem Directory Integrity

```bash
cargo run --bin foundry-cli -- system validate
```

### 4. Build Unified Release Package (Infra + Custom Subsystems)

```bash
./scripts/build-release.sh
```

---

## 📖 Documentation

- 🌐 [Official Documentation (GitHub Pages)](https://foundkit.github.io/foundry/)
- 📘 [Getting Started Guide](docs/src/content/docs/getting-started.md)
- 🏗️ [Architecture Blueprint](docs/src/content/docs/architecture/blueprint.md)
- 🔌 [Subsystem Extensions & Admin UI Guide](docs/src/content/docs/guides/extensions.md)
- 🗺️ [Development Roadmap & TODO](docs/src/content/docs/roadmap.md)

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

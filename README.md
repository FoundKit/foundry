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

1. **Multi-Tenant Sub-System Engine**: Run $N$ independent sub-systems (apps, mini-programs, campaign backends) within a single Foundry instance with strict tenant isolation.
2. **Zero-DDL Dynamic Storage Engine**: Visually define system configs and dynamic data models without database DDL migrations or metadata locks.
3. **Automatic RESTful CRUD**: Instant REST APIs generated for all dynamic models (`/api/v1/s/{system_slug}/{model_slug}`).
4. **Decoupled RESTful Route Architecture**: Standardized REST paths separating Admin API (`/api/v1/admin/*`), Auto-CRUD (`/api/v1/s/{slug}/*`), and Custom Extension APIs (`/api/v1/s/{slug}/ext/*`).
5. **Standalone External Subsystem Hosting**: Subsystem code and static UI assets can be hosted in separate Git repositories, dynamically discovered via `FOUNDRY_SYSTEMS_DIR` or `./external_systems`.
6. **Subsystem Custom Admin UI Pages & SDK Bridge**: Register custom admin pages seamlessly integrated into the Foundry Admin UI shell with automatic JWT token and theme injection via `FoundryBridge`.
7. **Hierarchical Admin IAM & Topic-Scoped RBAC**: Super Admin (`super_admin`) and scoped Topic Admins (`allowed_systems: ["vip_mall"]`).
8. **Non-GET Operation Audit Trail**: Asynchronous logging of all state-mutating requests (POST, PUT, PATCH, DELETE, login).

---

## 📁 Repository Structure

```
foundry/
├── apps/
│   ├── server/           # Foundry Core Server (Axum binary)
│   ├── admin/            # Visual Admin Dashboard SPA (React + TypeScript + Tailwind)
│   └── cli/              # Developer CLI Tool (Subsystem scaffolding & admin utility)
├── crates/
│   ├── foundry_core/     # SystemContext, SubsystemModule, CustomAdminPageSpec
│   ├── foundry_storage/  # PostgreSQL Zero-DDL storage & dynamic model ORM
│   ├── foundry_auth/     # Admin IAM, Argon2id, JWT & Topic RBAC
│   ├── foundry_engine/   # Multi-system router, Auto-CRUD, Audit middleware
│   └── foundry_extension/# Mutation hooks & WASM extension pipeline
├── systems/              # Sub-System compiled code workspace
├── external_systems/     # Standalone external subsystem repositories
├── migrations/           # Baseline database initialization (`init.sql`)
└── docker/               # Docker container configurations
```

---

## 🚀 Quick Scaffolding Commands

### 1. Scaffold a Compiled Subsystem (Monorepo)

```bash
cargo run --bin foundry-cli -- system new carnival_demo --name "Carnival Demo"
```

### 2. Scaffold a Standalone External Subsystem (Decoupled Repository)

```bash
cargo run --bin foundry-cli -- system new-external vip_mall --name "VIP Mall Standalone"
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

# Foundry

<div align="center">

**Build complete systems from a common foundation.**

*Foundry is an open-source platform for building and running multiple independent backend systems from a shared foundation.*

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-2024%20%2F%201.85%2B-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18%2B%20%2F%2019-61dafb.svg)](https://react.dev)
[![PostgreSQL](https://img.shields.io/badge/postgresql-18.6%2B-336791.svg)](https://www.postgresql.org)
[![Redis](https://img.shields.io/badge/redis-8%2B-dc382d.svg)](https://redis.io)

</div>

---

## 🌟 Key Capabilities

1. **Multi-Tenant Sub-System Engine**: Run and manage $N$ independent sub-systems (apps, mini-programs, campaign backends) within a single Foundry instance with strict tenant isolation.
2. **Zero-DDL Dynamic Storage**: Administrators can visually define system configs and dynamic data models without running database DDL or altering production tables.
3. **Automatic RESTful CRUD**: Instant REST APIs generated for all dynamic models with filtering, sorting, and pagination.
4. **Code-First Sub-System Extensibility**: Dedicated 3-layer architecture (`controllers/`, `logic/`, `dto/`) in `systems/src/<slug>/` for autonomous sub-system code.
5. **Admin IAM & Topic-Scoped RBAC**: Super Admin (`super_admin`) and scoped Topic Admins (`allowed_systems: ["carnival_2026"]`).
6. **Non-GET Operation Audit Trail**: Full recording of all state-mutating HTTP requests (POST, PUT, PATCH, DELETE, login) with headers, query parameters, body payloads, and dynamic action names.
7. **REST-First & OpenAPI-Native**: Native OpenAPI 3.0 generation at `/api/v1/openapi.json` with Swagger UI at `/api/v1/docs`.

---

## 📁 Repository Structure

```
foundry/
├── apps/
│   ├── server/           # Foundry Core Server (Axum binary)
│   ├── admin/            # Visual Admin Dashboard SPA (React + TypeScript + Tailwind)
│   └── cli/              # Developer CLI Tool (System scaffolding & migrations)
├── crates/
│   ├── foundry_core/     # SystemContext, SubsystemModule trait, error envelopes
│   ├── foundry_storage/  # PostgreSQL Zero-DDL storage & dynamic model ORM
│   ├── foundry_auth/     # Admin IAM, Argon2id, JWT & Topic RBAC
│   ├── foundry_engine/   # Multi-system router, Auto-CRUD, Audit middleware
│   └── foundry_extension/# Mutation hooks & WASM extension pipeline
├── systems/              # Sub-System custom code workspace
│   ├── src/lib.rs        # Subsystem registry loader
│   └── src/carnival_demo/# Sample built-in 3-layer sub-system
├── migrations/           # Zero-DDL database schema (`init.sql`)
└── docker/               # Dockerfile & Docker Compose
```

---

## 🚀 Getting Started

### 1. Local Infrastructure with Docker Compose

```bash
cd docker
docker compose up -d postgres redis
```

### 2. Run Database Migrations & Seed Default Super Admin

```bash
cargo run --bin foundry-cli -- migrate
```

Default credentials:
- **Username**: `admin`
- **Password**: `admin123456`

### 3. Start Foundry Server

```bash
cargo run --bin foundry-server
```

Server will be running at `http://localhost:8080`.

### 4. Start Admin Dashboard (Frontend SPA)

```bash
cd apps/admin
pnpm install
pnpm dev
```

Dashboard will be running at `http://localhost:3000` (see [`apps/admin/README.md`](apps/admin/README.md) for quality checks and details).

### 5. Scaffold a New Sub-System

```bash
cargo run --bin foundry-cli -- system new vip_mall --name "VIP Mall 2026"
```

---

## 📖 Documentation

- [Architecture Blueprint](docs/architecture.md)
- [Development Roadmap & TODO](docs/todo.md)

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

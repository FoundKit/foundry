---
title: Getting Started
description: Quick start guide to running and developing with Foundry.
---

# Getting Started with Foundry

Foundry is an open-source multi-system backend platform built with Rust, PostgreSQL, Redis, and React, featuring complete decoupling between platform infrastructure and custom business subsystems.

## Prerequisites

Before starting, ensure you have the following installed:

- **Rust 1.85+ / 2024 edition**
- **Node.js 20+** and **pnpm 9+**
- **Docker & Docker Compose** (for PostgreSQL and Redis)

---

## 1. Start Infrastructure Services

Use Docker Compose to start local PostgreSQL and Redis instances:

```bash
docker compose -f docker/docker-compose.yml up -d
```

Verify the services are running:

- PostgreSQL: `localhost:5432` (database: `foundry`, user: `postgres`, password: `postgrespassword`)
- Redis: `localhost:6379`

---

## 2. Initialize the Database

Apply the baseline database migration:

```bash
cargo run --bin foundry-cli -- migrate
```

---

## 3. Run Foundry Server

Launch the Foundry Axum server:

```bash
cargo run --bin foundry-server
```

The server will start listening at `http://127.0.0.1:8080`.

---

## 4. Run Admin Dashboard SPA

Navigate to `apps/admin` and start the Vite dev server:

```bash
pnpm --dir apps/admin install
pnpm --dir apps/admin dev
```

Open your browser at `http://localhost:5173`. Default Super Admin credentials:

- **Username**: `admin`
- **Password**: `admin123456`

---

## 5. Scaffold a Self-Contained Subsystem

Use the `foundry-cli` tool to scaffold a self-contained subsystem with controllers, logic, DTOs, and custom admin pages:

```bash
cargo run --bin foundry-cli -- system new carnival_demo --name "Carnival Demo"
```

Or initialize a standalone Git repository for team custom subsystems:

```bash
cargo run --bin foundry-cli -- system init-repo ../my-foundry-systems
```

---

## 6. Unified Production Release Packaging

Merge platform infrastructure and all custom subsystems into a release bundle with a single command:

```bash
./scripts/build-release.sh
```

---

## Next Steps

- Explore the [Architecture Blueprint](../architecture/blueprint/) to understand Foundry's decoupled architecture.
- Read the [Subsystem Extensions Guide](../guides/extensions/) to write custom Rust endpoints and Admin UI pages.
- Check the [Development Roadmap](../roadmap/) for upcoming features.

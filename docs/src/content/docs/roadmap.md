---
title: Development Roadmap
description: Foundry development milestones, completed features, and upcoming roadmap.
---

# Foundry Development Roadmap & TODO List

> **Project**: `foundry`  
> **Organization**: [foundkit](https://github.com/foundkit)  
> **Repository**: Monorepo (Rust Engine + Sub-Systems Workspace + Web Admin UI + CLI)

---

## 📌 Legend & Status Tracker
- [ ] **Pending**
- [/] **In Progress**
- [x] **Completed**

---

## Milestone 0: Monorepo Foundation & Workspace Setup

- [x] **0.1 Repository & Project Scaffolding**
  - [x] Initialize Cargo Workspace for Rust backend crates and `systems/` workspace (`Cargo.toml`).
  - [x] Apply baseline database schema migration from `migrations/init.sql` (Zero-DDL tables).
  - [x] Initialize frontend project for Admin SPA in `apps/admin` (React + Vite + TypeScript + Tailwind CSS).
  - [x] Configure GitHub Actions CI for linting, testing, and formatting.
  - [x] Setup `docker/` container configurations (`docker-compose.yml` for PostgreSQL 18.6+ + Redis 8+).

---

## Milestone 1: Core Engine & Multi-System Isolation (Backend MVP)

- [x] **1.1 System Context & Tenant Routing (`crates/foundry_core` & `crates/foundry_engine`)**
  - [x] Implement Sub-System metadata primitives: `system_id` (UUID) and immutable unique `system_slug`.
  - [x] Build Axum middleware to extract and validate `SystemContext`.
  - [x] Parse `Accept-Language` header into `SystemContext.locale`.
  - [x] Implement global error handling with i18n JSON envelope.

- [x] **1.2 Zero-DDL Storage Engine & Model Runtime (`crates/foundry_storage`)**
  - [x] Implement System Configs engine (`system_configs`).
  - [x] Implement Dynamic Data Models engine (`models` & `model_fields`).
  - [x] Build Data Model runtime ORM on `model_records` (`ctx.model("{model_slug}")`).
  - [x] Programmatic Configs API (`ctx.configs()`).
  - [x] Tenant namespace Redis caching (`foundry:{system_slug}:*`).

- [x] **1.3 Auto-CRUD & System Configs API Engine (`crates/foundry_engine`)**
  - [x] Dynamic Data Model Auto-CRUD endpoints.
  - [x] System Configs endpoints.

- [x] **1.4 Admin IAM & Topic-Scoped RBAC (`crates/foundry_auth`)**
  - [x] Admin identity and Argon2id password hashing.
  - [x] Super Admin (`super_admin`) and scoped Topic Admins (`admin`).
  - [x] JWT token lifecycle.

---

## Milestone 2: Web Admin Dashboard (`apps/admin`)

- [x] **2.1 Dashboard Infrastructure & i18n**
  - [x] React + Vite + Tailwind CSS design system.
  - [x] Multilingual dictionary support (`react-i18next`).
  - [x] Two-tier layout: Platform Control Plane vs. Subsystem Dedicated Workspaces.
  - [x] Bidirectional URL route persistence and deep linking.

---

## Milestone 3: Non-GET Write Audit Engine

- [x] **3.1 Non-GET Audit Interceptor Middleware (`crates/foundry_engine`)**
  - [x] Intercept state-mutating requests (POST, PUT, PATCH, DELETE, login).
  - [x] Discrete storage: `headers`, raw `query_params`, raw `body_params`.
  - [x] Dynamic action name resolver.

---

## Milestone 4: Subsystem Complete Decoupling & Unified Release Packaging

- [x] **4.1 Decoupled Two-Repository Architecture**
  - [x] Complete separation between base infrastructure platform and custom subsystem repositories.
  - [x] Self-contained subsystem standard: Controllers, Domain Logic, DTOs, Custom Admin Pages (`custom_pages`), and Manifest (`subsystem.json`).
  - [x] Zero merge conflicts during upstream base engine updates (`git pull upstream`).

- [x] **4.2 Subsystem Custom Admin UI Pages & SDK Bridge**
  - [x] Automatic static custom page hosting at `/api/v1/s/{slug}/ext/custom-pages/*`.
  - [x] `FoundryBridge` postMessage protocol for token and theme sync.
  - [x] Granular role access enforcement (`required_role`).

- [x] **4.3 Developer CLI Scaffolding & Validation Tools (`apps/cli`)**
  - [x] `foundry-cli system init-repo`: Initialize standalone subsystem Git repository.
  - [x] `foundry-cli system new`: Scaffold self-contained subsystem.
  - [x] `foundry-cli system validate`: Validate directory structure and manifest integrity.

- [x] **4.4 Unified Production Release Packaging Pipeline**
  - [x] `scripts/build-release.sh` automated release script.
  - [x] Multi-stage Dockerfile combining base platform and custom subsystems.

---

## 🔮 Upcoming Roadmap

- [/] **Wasmtime Sandboxed Plugin Support**
- [/] **Live OpenAPI 3.0 Aggregator & Swagger UI**
- [ ] **High-Availability Multi-Node Cluster Orchestration**

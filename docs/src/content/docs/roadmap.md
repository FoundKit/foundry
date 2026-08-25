---
title: Development Roadmap & Versioning
description: Foundry development milestones, completed features, versioning strategy, and upcoming roadmap.
---

# Foundry Roadmap & Versioning Strategy

> **Project**: `foundry`  
> **Organization**: [foundkit](https://github.com/foundkit)  

---

## 📌 Status Legend
- [ ] **Pending**
- [/] **In Progress**
- [x] **Completed**

---

## Milestone 1: Framework Decoupling & Facade Architecture (`v0.1.0`)

- [x] **1.1 Core Platform & Application Decoupling**
  - [x] Create facade crate `foundry` with `FoundryApp::builder()` and unified prelude.
  - [x] Full separation between framework runtime and independent user applications.
  - [x] Package and distribute via standard Cargo dependencies (`foundry = "0.1"`).

- [x] **1.2 Subsystem Standard & Dynamic Routing**
  - [x] `SubsystemModule` trait for registering business modules, custom routes, and admin pages.
  - [x] Static and dynamic external subsystem discovery engine.
  - [x] Topic-scoped RBAC and Context extraction.

- [x] **1.3 Storage & Admin Control Plane**
  - [x] PostgreSQL Zero-DDL dynamic model schema engine.
  - [x] Auto-generated RESTful CRUD endpoints (`/api/v1/s/{system_slug}/{model_slug}`).
  - [x] Embedded Admin SPA Shell with iframe sandbox bridge (`FoundryBridge`).

- [x] **1.4 Developer CLI Tooling (`foundry-cli`)**
  - [x] `foundry new <project>`: Scaffold standalone user application.
  - [x] `foundry system new <slug>`: Scaffold business subsystem.
  - [x] `foundry migrate`: Apply baseline migrations.
  - [x] `foundry admin create` / `reset-password`: Administrator IAM tooling.

- [x] **1.5 Reference Example & Automated Testing**
  - [x] `examples/blog_platform`: Full-featured standalone reference application.
  - [x] End-to-end integration and CLI smoke tests in CI.

---

## Milestone 2: Extensibility & Developer Experience (`v0.2.0`)

- [/] **2.1 Sandboxed WebAssembly (Wasmtime) Plugin Runtime**
  - [ ] Hot-reloadable WASM subsystem modules.
  - [ ] Safe sandbox boundary for multi-tenant extensions.

- [/] **2.2 OpenAPI 3.0 & Swagger UI Integration**
  - [ ] Auto-generate OpenAPI schemas for dynamically created data models and subsystem routes.
  - [ ] Embedded Swagger UI explorer in Admin Shell.

- [ ] **2.3 Multi-Database Support (MySQL, SQLite)**
  - [ ] Pluggable SQL dialect adapters for Zero-DDL dynamic models.

---

## Milestone 3: Enterprise & High-Availability (`v1.0.0`)

- [ ] **3.1 Cluster Orchestration & Distributed Pub/Sub**
  - [ ] Real-time WebSocket event streaming.
  - [ ] Multi-node cache synchronization via Redis Pub/Sub.

- [ ] **3.2 Dynamic Webhook Dispatcher**
  - [ ] Configurable webhook endpoints triggered on entity mutations.
  - [ ] Retry queue with exponential backoff.

---

## 🏷️ Versioning Strategy & Upgrade Guide

Foundry follows **Semantic Versioning (SemVer)**:
- **Major versions (`1.x.x`)**: Breaking changes to public APIs with automated migration guides.
- **Minor versions (`0.x.x`, `1.x.0`)**: New feature additions, builder extensions, and backward-compatible trait defaults.
- **Patch versions (`0.1.x`)**: Backward-compatible bug fixes and internal performance enhancements.

Upgrading in a user application:
```bash
cargo update -p foundry
```

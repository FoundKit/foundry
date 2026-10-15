---
title: System Development Tutorial Overview & Roadmap
description: Complete modular Foundry system development tutorial. Chapter breakdown, conceptual mappings, and learning roadmap.
---

# Foundry System Development Tutorial Overview

Foundry adheres to **"total framework-application decoupling"**, **"convention over configuration"**, and **"high cohesion with loose coupling"**:
* **Independent User Applications**: Build inside your own independent Git repository, consume Foundry via standard Cargo dependencies, and upgrade upstream with zero merge conflicts.
* **Unified Facade**: Single import `use foundry::prelude::*;` brings routing, parameter extractors, ORM query builders, response envelopes, and logging macros.
* **Dual Storage Modes**: Code-First strongly-typed ORM for core transactional business, plus Zero-DDL dynamic models for schema-free rapid iteration.
* **Out-of-the-Box Admin Console**: All subsystems seamlessly integrate into an embedded React Admin console with sandboxed views and postMessage authentication.

This tutorial decouples system development into 11 focused chapters, providing a clear and structured guide for building enterprise-grade backend systems on Foundry.

---

## 🧭 Tutorial Chapters (Modular Structure)

```mermaid
flowchart TD
    subgraph S1["1. Foundations & Architecture"]
        C1["01. Architecture & Request Lifecycle"]
        C2["02. Routing System & Namespaces"]
    end

    subgraph S2["2. Request Handling Layer"]
        C3["03. Controllers & Response Envelope"]
        C4["04. Request Validation & DTOs"]
    end

    subgraph S3["3. Data Persistence"]
        C5["05. Database Setup & Code-First ORM"]
        C6["06. Zero-DDL Dynamic Models & Auto-CRUD"]
    end

    subgraph S4["4. Security & Extensions"]
        C7["07. Authentication & Hierarchical RBAC"]
        C8["08. Lifecycle Mutation Hooks"]
        C9["09. Admin Console & postMessage SDK"]
    end

    subgraph S5["5. Walkthrough & Production"]
        C10["10. Hands-on Walkthrough: E-Commerce"]
        C11["11. Architectural Best Practices"]
    end

    S1 --> S2 --> S3 --> S4 --> S5
```

### Chapter Quick Index

| Chapter | Title | Core Focus | Key Components |
|:---:|---|---|---|
| **01** | [Architecture & Request Lifecycle](./01-architecture-lifecycle/) | Decoupling model, HTTP sequence diagram, Facade | Architecture Overview, Lifecycle, `foundry::prelude` |
| **02** | [Routing System & Namespaces](./02-routing/) | Unified namespace table, subsystem route registration | Route Definition, Namespaces, Method Binding |
| **03** | [Controllers & Response Envelope](./03-controllers/) | Axum Handlers, Extractors, `ApiResponse`, `AppError` | Controller Handlers, Extractors, Unified Output |
| **04** | [Request Validation & DTOs](./04-validation/) | `#[derive(Validate)]`, rule reference, HTTP 422 | Validator DTOs, Compile-time Rules, Guard |
| **05** | [Database Setup & Code-First ORM](./05-database-orm/) | Multi-DB SPI, EntitySchema, fluent ORM builder, soft deletes | Multi-DB SPI, EntitySchema, Query Builder |
| **06** | [Zero-DDL Dynamic Models & Auto-CRUD](./06-dynamic-models/) | Schema-free models, `RecordStore` API, auto REST endpoints | Dynamic Models, RecordStore, Auto-CRUD |
| **07** | [Authentication & Hierarchical RBAC](./07-auth-security/) | IAM, Argon2id, JWT, 3-tier RBAC tenant isolation | IAM, Argon2id, JWT, RBAC Guards |
| **08** | [Lifecycle Mutation Hooks](./08-mutation-hooks/) | `MutationHook` pipeline, 6 lifecycle events, audit logging | Pre/Post Lifecycle Hooks, Invalidation |
| **09** | [Admin Console & postMessage SDK](./09-admin-integration/) | React Admin shell, sandboxed custom pages, postMessage | Admin Shell, Sandboxed Views, Auth Protocol |
| **10** | [Walkthrough: E-Commerce System](./10-walkthrough-shop/) | End-to-end tutorial: DTO, Service, Controller, Subsystem, Test | Practical E-Commerce Implementation & Tests |
| **11** | [Architectural Best Practices](./11-best-practices/) | Thin Controller/Fat Service, pool tuning, Docker production | Layering Rules, Performance, Docker Deployment |

---

## 🎯 Web Architecture Concepts to Foundry Mapping

| Standard Web Concept | Foundry (Rust) Equivalent | Key Advantages |
|---|---|---|
| **Application Module** | **`src/systems/<slug>/` Subsystem** | High cohesion, strict domain boundary |
| **Module Route Registration** | **`SubsystemModule::register_routes`** | Auto-prefixed under `/api/v1/s/<slug>/ext/` |
| **Controller Handlers** | **Axum Handler Functions** | Fearless concurrency, zero object overhead |
| **Request Input Extraction** | **`Query` / `Json` / `Path` Extractors** | Compile-time strong deserialization |
| **Validator DTOs** | **`#[derive(Validate)]` DTOs** | Compile-time validation annotations |
| **Relational Query Builder** | **`OrmSelect::from_table()->...`** | Multi-dialect parameter binding (`$1` vs `?`) |
| **Insert Builder** | **`OrmInsert::into_table()->...`** | Unified SQL injection prevention |
| **Pipeline Middleware** | **Tower / Axum Middleware Layers** | High throughput request tracing & audit |
| **Lifecycle Hooks / Events** | **`MutationHook` Traits** | Pre/post mutation interceptor pipeline |

---

## 🚀 Recommended Learning Path

Begin your journey with [01. Architecture & Request Lifecycle](./01-architecture-lifecycle/)!

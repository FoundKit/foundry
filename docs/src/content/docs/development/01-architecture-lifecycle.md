---
title: Architecture & Request Lifecycle
description: Deep dive into Foundry's decoupled platform architecture, bootstrapping process, FoundryApp Builder, and the full HTTP request lifecycle.
---

# Architecture & Request Lifecycle

This chapter covers:
1. Foundry's "framework-application complete decoupling" philosophy
2. Application bootstrapping with `FoundryApp::builder()`
3. Full HTTP request lifecycle tracing
4. Component layering and separation of concerns

---

## 1. Architectural Layering

Foundry leverages Rust's memory safety and high-concurrency runtime to partition systems into a framework layer and an application layer:

```text
Foundry Framework Layer (crates/foundry, crates/foundry_core, crates/foundry_engine...)
    │  Provides generic infrastructure: Axum engine, response envelopes, multi-DB SPI, RBAC, Admin
    │  Consumed via standard Cargo dependency
    ▼
User Application (Standalone Git Repository)
    ├── src/main.rs             # Bootstrapping & service launch entry point
    └── src/systems/            # Domain subsystems directory
        ├── shop/               # E-commerce subsystem (controllers, logic, DTOs, custom admin UI)
        └── billing/            # Billing subsystem
```

### Component Responsibility Table

| Component | Foundry Implementation | Description & Responsibility |
|---|---|---|
| **Application Entry Point** | **`src/main.rs` (`#[foundry::main]`)** | Bootstrapping entry point, reads `.env`, invokes `FoundryApp::builder()` |
| **Business Subsystem** | **`SubsystemModule` trait** | High-cohesion, self-contained business domain package |
| **Unified Facade** | **`use foundry::prelude::*;`** | Re-exports standard primitives, extractors, and macros |
| **State Injection** | **`Extension<SystemContext>` / `AppState`** | State injection with zero runtime reflection overhead |
| **Route Registration** | **`SubsystemModule::register_routes`** | Mounted automatically under subsystem namespace |
| **Controller Handlers** | **Axum Handler Function** | Async function extracting params and returning `ApiResponse` |
| **Request Validator** | **`#[derive(Validate)]` DTO** | Compile-time validation annotations, rejecting malformed input |
| **ORM Schemas & Queries** | **`EntitySchema` / `OrmSelect`** | Code-First schemas and dialect-adaptive query builders |
| **Lifecycle Hooks** | **`MutationHook` Pipeline** | Lifecycle pre/post mutation hooks (`before_create`, `after_create`) |

---

## 2. Full HTTP Request Lifecycle

```mermaid
sequenceDiagram
    autonumber
    actor Client as Client (SPA / Mobile / Third-party)
    participant Server as HTTP Transport (Axum)
    participant MW as Global Middleware (Trace / CORS / Context / Audit)
    participant Router as Engine Router
    participant Auth as RBAC Guard
    participant Handler as Controller Handler
    participant DTO as Validator DTO
    participant Logic as Domain Logic Service
    participant Hook as MutationHook Pipeline
    participant DB as Storage (Postgres/MySQL/MariaDB/Mongo/Redis)

    Client->>Server: HTTP Request (POST /api/v1/s/shop/ext/orders)
    Server->>MW: 1. Pass through middleware (extract SystemContext, trace duration)
    MW->>Router: 2. Match route pattern (/api/v1/s/{subsystem}/ext/*)
    Router->>Auth: 3. Verify JWT & Topic-scoped RBAC permissions
    Auth-->>Router: Authorized (inject AdminClaims / allow public)
    Router->>Handler: 4. Dispatch to subsystem handler
    Handler->>DTO: 5. Deserialize & execute payload.validate()?
    alt Validation Failed
        DTO-->>Handler: Validation Error
        Handler-->>Client: HTTP 422 ApiResponse::error("VALIDATION_ERROR")
    else Validation Passed
        Handler->>Logic: 6. Invoke OrderService::create_order(...)
        Logic->>Hook: 7. Trigger before_create hook (enrichment / security)
        Hook-->>Logic: Hook approved
        Logic->>DB: 8. Execute OrmInsert / transaction persistence
        DB-->>Logic: Row persisted
        Logic->>Hook: 9. Trigger after_create hook (notify / invalidate cache)
        Hook-->>Logic: Post-action finished
        Logic-->>Handler: Return domain response struct
        Handler-->>Client: HTTP 200/201 ApiResponse::success(data)
    end
    MW->>DB: 10. Persist audit log asynchronously (AuditStore)
```

---

## 3. Application Bootstrapping (`main.rs`)

Foundry uses a fluent Builder API to configure and start services:

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::shop::ShopSubsystem;

#[foundry::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read configuration from environment (.env)
    let config = FoundryConfig::from_env();

    // 2. Build Foundry Application instance
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(ShopSubsystem)
        // .register_hook(ShopAuditHook)
        .build()
        .await?;

    info!("🚀 Server is ready and listening...");

    // 3. Start HTTP server loop
    app.run().await?;
    Ok(())
}
```

---

## 4. Unified Facade (`prelude`)

With a single import:
```rust
use foundry::prelude::*;
```
Your code gets access to:
* **Routing & Extractors**: `Router`, `Json`, `Query`, `Path`, `Extension`, `get`, `post`, `put`, `delete`
* **Response & Error Envelopes**: `ApiResponse`, `PaginatedData`, `AppResult`, `AppError`
* **ORM Query Builders**: `OrmSelect`, `OrmInsert`, `OrmUpdate`, `OrmDelete`, `Dialect`
* **Structured Logging**: `info!`, `warn!`, `error!`, `debug!`, `trace!`
* **Subsystems & Schemas**: `SubsystemModule`, `SystemContext`, `AppState`, `EntitySchema`, `ColumnDef`, `IndexDef`

Next chapter: [Routing System & Namespaces](./02-routing/)

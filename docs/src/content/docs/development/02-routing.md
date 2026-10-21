---
title: Routing System & Namespaces
description: Master Foundry's unified namespace routing, subsystem extension routing, HTTP verb binding, and path parameter extraction.
---

This chapter covers:
1. Foundry's global routing layout and unified namespace
2. Registering custom business routes in `SubsystemModule`
3. Understanding `/ext/*` custom routes vs Auto-CRUD routes
4. Binding HTTP methods and extracting path parameters

---

## 1. Global Unified Routing Space

Foundry enforces an unambiguous routing namespace. This architecture guarantees **zero route collisions** across multiple business subsystems:

```text
/ (Root)
├── /health                               # Container liveness health check (returns OK)
├── /admin/                               # Embedded React Admin Console SPA
│   └── /ext/{subsystem}/*                # Subsystem custom admin pages (static HTML / iframe)
│
└── /api/                                 # Unified Platform API Space
    ├── /health                           # API health check (returns OK)
    ├── /admin/*                          # Admin & management APIs (auth, audit, systems)
    │   └── /ext/{subsystem}/*            # Subsystem admin management APIs (via register_admin_routes, protected by auth)
    │
    ├── /ext/{subsystem}/*                # Subsystem public extension APIs (via register_routes)
    │
    └── /s/{subsystem}/                   # Subsystem Auto-CRUD space (/s/shop/, /s/blog/)
        ├── /configs                      # Aggregated key-value configs
        └── /{model}                      # Auto-CRUD engine routes (zero code required)
            ├── GET    /api/s/{subsystem}/{model}      (List & paginate)
            ├── POST   /api/s/{subsystem}/{model}      (Create entity)
            ├── GET    /api/s/{subsystem}/{model}/{id} (Get entity)
            ├── PUT    /api/s/{subsystem}/{model}/{id} (Update entity)
            └── DELETE /api/s/{subsystem}/{model}/{id} (Delete entity)
```

---

## 2. Subsystem Route Registration (`register_admin_routes` & `register_routes`)

Implement `SubsystemModule` in `src/systems/<slug>/mod.rs` and override `register_admin_routes` for management endpoints and `register_routes` for public endpoints:

```rust
// src/systems/shop/mod.rs
pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;

pub struct ShopSubsystem;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str { "shop" }
    fn display_name(&self) -> &'static str { "E-Commerce Shop" }

    // Mounts admin management handlers under /api/admin/ext/shop/* (protected by admin auth & topic access)
    fn register_admin_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_admin_routes())
    }

    // Mounts public handlers under /api/ext/shop/*
    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_public_routes())
    }
}
```

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str {
        "shop"
    }

    fn display_name(&self) -> &'static str {
        "E-Commerce Shop"
    }

    fn description(&self) -> &'static str {
        "Product catalog, checkout, and order lifecycle management"
    }

    // Mounted under /api/s/shop/ext/*
    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_routes())
    }
}
```

---

## 3. Route Definition & Method Binding

In `src/systems/shop/controllers/mod.rs`:

```rust
use foundry::prelude::*;

pub fn build_routes() -> Router {
    Router::new()
        .route("/orders", post(create_order).get(list_orders))
        .route("/orders/{id}", get(get_order_by_id).put(update_order))
        .route("/orders/{id}/cancel", post(cancel_order))
}
```

### Handler Signatures

```rust
pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    Ok(ApiResponse::ok(data))
}

pub async fn get_order_by_id(
    Extension(ctx): Extension<SystemContext>,
    Path(id): Path<i64>,
) -> AppResult<ApiResponse<OrderResponse>> {
    Ok(ApiResponse::ok(data))
}
```

Next chapter: [Controllers & Response Envelope](./03-controllers/)

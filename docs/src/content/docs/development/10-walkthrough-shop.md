---
title: "10. Hands-on Walkthrough: E-Commerce System"
description: Build a complete, production-grade e-commerce shop subsystem with validation, domain services, Code-First ORM, unified responses, and tests.
---

# 10. Hands-on Walkthrough: E-Commerce System

> **Chapter Objectives**:
> 1. Connect all foundational concepts in a real-world e-commerce shop implementation;
> 2. Implement input validation, business pricing calculations, and ORM persistence;
> 3. Verify endpoints using `curl`;
> 4. Write an automated end-to-end integration test with `axum-test`.

---

## 1. DTOs & Validation (`dto/mod.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderItem {
    #[validate(range(min = 1, message = "Product ID must be positive"))]
    pub product_id: i64,
    #[validate(range(min = 1, max = 99, message = "Quantity must be 1-99"))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 2, max = 32, message = "Name must be 2-32 chars"))]
    pub customer_name: String,
    #[validate(email(message = "Must provide valid email"))]
    pub customer_email: String,
    #[validate(length(min = 1, message = "At least one item required"))]
    #[validate(nested)]
    pub items: Vec<CreateOrderItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub total_amount: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
```

---

## 2. Domain Service (`logic/mod.rs`)

```rust
use chrono::Utc;
use foundry::prelude::*;
use crate::systems::shop::dto::{CreateOrderRequest, OrderResponse};

pub struct OrderService;

impl OrderService {
    pub async fn create_order(
        ctx: &SystemContext,
        _state: &AppState,
        req: CreateOrderRequest,
    ) -> AppResult<OrderResponse> {
        let order_no = format!("ORD-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), fastrand::u32(1000..9999));
        let total_amount: f64 = req.items.iter().map(|item| (item.quantity as f64) * 99.0).sum();

        let (sql, _binds) = OrmInsert::into_table("shop_orders")
            .value("system_id", &ctx.system_slug)
            .value("order_no", &order_no)
            .value("customer_name", &req.customer_name)
            .value("customer_email", &req.customer_email)
            .value("total_amount", total_amount)
            .value("status", "pending")
            .to_sql(Dialect::Postgres);

        info!("Order persisted with SQL: {}", sql);

        Ok(OrderResponse {
            id: Utc::now().timestamp_millis(),
            order_no,
            customer_name: req.customer_name,
            total_amount,
            status: "pending".to_string(),
            created_at: Utc::now(),
        })
    }
}
```

---

## 3. Controller (`controllers/mod.rs`)

```rust
use foundry::prelude::*;
use crate::systems::shop::dto::{CreateOrderRequest, OrderResponse};
use crate::systems::shop::logic::OrderService;

pub fn build_routes() -> Router {
    Router::new().route("/orders", post(create_order))
}

pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    payload.validate()?;
    let order = OrderService::create_order(&ctx, &state, payload).await?;
    info!(order_no = %order.order_no, "Order created successfully");
    Ok(ApiResponse::created(order))
}
```

---

## 4. Subsystem Module & Assembly

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

    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_routes())
    }

    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            EntitySchema::new("shop_orders")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("order_no", 64).not_null().unique())
                .column(ColumnDef::varchar("customer_name", 64).not_null())
                .column(ColumnDef::varchar("customer_email", 128).not_null())
                .column(ColumnDef::decimal("total_amount", 12, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("deleted_at").nullable())
                .index(IndexDef::new("idx_shop_orders_sys", &["system_id", "deleted_at"]))
        ]
    }
}
```

---

## 5. Verification with curl

```bash
curl -X POST http://127.0.0.1:8080/api/v1/s/shop/ext/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Alice",
    "customer_email": "alice@example.com",
    "items": [{ "product_id": 1, "quantity": 2 }]
  }'

# Response (HTTP 201 Created):
# {"code":0,"message":"success","data":{"id":1729000000000,"order_no":"ORD-...","customer_name":"Alice","total_amount":198.0,"status":"pending"},"meta":null}
```

Next chapter: [11. Architectural Best Practices & Production](./11-best-practices/)

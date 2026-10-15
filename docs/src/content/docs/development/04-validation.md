---
title: 04. Request Validation & DTOs
description: Master compile-time request validation DTOs in Foundry, validator macro annotations, common rule reference, and HTTP 422 short-circuiting.
---

# 04. Request Validation & DTOs

> **Chapter Objectives**:
> 1. Understand defensive validation principles;
> 2. Implement DTOs with `#[derive(Validate)]`;
> 3. Use common validation rules: length, email, range, regex, and nested;
> 4. Short-circuit invalid requests into clean HTTP 422 responses.

---

## 1. Validation Philosophy

Foundry binds validation directly to Rust's type system via declarative annotations:

```rust
// src/systems/shop/dto/mod.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderItem {
    #[validate(range(min = 1, message = "Product ID must be positive"))]
    pub product_id: i64,

    #[validate(range(min = 1, max = 99, message = "Quantity must be between 1 and 99"))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 2, max = 32, message = "Name must be 2-32 chars"))]
    pub customer_name: String,

    #[validate(email(message = "Must provide a valid email"))]
    pub customer_email: String,

    #[validate(length(min = 1, message = "Order must have at least one item"))]
    #[validate(nested)]
    pub items: Vec<CreateOrderItem>,
}
```

---

## 2. One-Line Controller Validation

```rust
pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 1. One-line validation check: returns HTTP 422 automatically if invalid
    payload.validate()?;

    // 2. Guaranteed safe payload
    let order = OrderService::create_order(&ctx, &state, payload).await?;
    Ok(ApiResponse::created(order))
}
```

Next chapter: [05. Database Setup & Code-First ORM](./05-database-orm/)

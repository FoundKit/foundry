---
title: Controllers & Response Envelope
description: Write clean Axum controllers, extract request payloads, record structured logs, and return standardized ApiResponse and AppError envelopes.
---

This chapter covers:
1. Designing thin async Axum controller handlers
2. Extracting inputs with `Json`, `Query`, `Path`, and `Extension`
3. Mastering the unified `ApiResponse<T>` envelope
4. Understanding automatic HTTP status code mapping with `AppError`

---

## 1. Controller Layer Design Principles

Foundry promotes the **"Thin Controller"** paradigm:
* **Controller Responsibilities**: Request parameter extraction, DTO validation, invoking domain services, and returning `ApiResponse`.
* **Not in Controllers**: Heavy business calculations, cross-model transactions, or raw database queries. Push these down into domain services (`src/systems/<slug>/logic/`).

---

## 2. Request Parameter Extractors

Axum uses type-driven extractors declared directly in the handler signature:

```rust
use foundry::prelude::*;

pub async fn handle_sample_action(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<OrderQuery>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // Business logic...
}
```

---

## 3. Standardized Response Envelope (`ApiResponse<T>`)

```json
{
  "code": 0,
  "message": "success",
  "data": { ... },
  "meta": null
}
```

```rust
use foundry::prelude::*;

// Success responses
ApiResponse::ok(data)
ApiResponse::created(new_order)
ApiResponse::paginated(items, page, page_size, total_count)
```

> [!TIP]
> `ApiResponse<T>` implements Axum's `IntoResponse` for all `T: Serialize`. You can return `Ok(ApiResponse::ok(data))` directly without wrapping in `Json(...)`.

---

## 4. Error Handling (`AppError` & `AppResult`)

```rust
pub type AppResult<T> = Result<T, AppError>;
```

| Error Variant | Cause | HTTP Status |
|---|---|---|
| `AppError::Validation(msg)` | DTO validation failure | `422 Unprocessable Entity` |
| `AppError::NotFound(msg)` | Entity does not exist | `404 Not Found` |
| `AppError::Unauthorized(msg)` | Missing/expired JWT | `401 Unauthorized` |
| `AppError::Forbidden(msg)` | Insufficient permissions | `403 Forbidden` |
| `AppError::Conflict(msg)` | Unique key constraint violation | `409 Conflict` |
| `AppError::Database(msg)` | Storage driver error | `500 Internal Server Error` |

Next chapter: [Request Validation & DTOs](./04-validation/)

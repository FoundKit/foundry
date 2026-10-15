---
title: 03. 控制器、参数与响应规范
description: 学习如何在 Foundry 中编写清晰精简的 Axum 控制器、提取请求参数、记录结构化日志并返回统一标准的 ApiResponse 与 AppError。
---

# 03. 控制器、参数与响应规范

> **本章目标**：
> 1. 掌握 Axum 控制器 Handler 异步函数的设计与编写；
> 2. 熟练使用 `Json`、`Query`、`Path`、`Extension` 提取各类入参；
> 3. 掌握统一标准响应 `ApiResponse<T>` 的使用方式；
> 4. 理解全局错误信封 `AppError` 与 HTTP 状态码的自动映射机制。

---

## 1. 控制器定位与设计理念

在 Foundry 中，我们极力提倡 **“精简控制器 (Thin Controller)”** 的设计原则：
* **控制器的职责**：参数反序列化、入参格式校验（调用 DTO 校验）、调用领域逻辑服务（Service/Logic）、组装并返回统一响应信封 `ApiResponse`；
* **控制器不应做的事**：不要在控制器中直接堆叠复杂业务算法、跨表一致性逻辑或拼装底层 SQL 查询。这些代码应当下沉到独立的服务层（如 `src/systems/<slug>/logic/`）。

---

## 2. 请求参数提取器 (Extractors)

Axum 采用基于类型的参数提取器。只需在 Handler 函数的参数列表中按需声明，框架会在请求到达时自动解析并注入：

```rust
use foundry::prelude::*;

pub async fn handle_sample_action(
    // 1. 系统上下文 (包含当前子系统标识 ctx.system_slug)
    Extension(ctx): Extension<SystemContext>,

    // 2. 全局应用状态 (包含数据库门面 state.db 与 Redis state.redis)
    Extension(state): Extension<AppState>,

    // 3. 路径参数 (如 /orders/{id})
    Path(id): Path<i64>,

    // 4. URL 查询字符串参数 (如 ?page=1&limit=20&status=paid)
    Query(query): Query<OrderQuery>,

    // 5. JSON 请求体 (反序列化为强类型 DTO 结构体)
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 处理业务...
}
```

### 常用提取器速查

| 提取器 | 作用 | 示例 |
|---|---|---|
| `Extension<SystemContext>` | 获取当前请求所在的子系统租户上下文 | `ctx.system_slug` |
| `Extension<AppState>` | 获取全局共享单例（数据库连接池、Redis 连接池） | `state.db`, `state.redis` |
| `Json<T>` | 解析 POST/PUT 请求体为 JSON 结构体 | `Json(payload)` |
| `Query<T>` | 解析 URL 查询参数（`?page=1&limit=20`） | `Query(query)` |
| `Path<T>` | 解析 URL 路由中的路径变量（`/{id}`） | `Path(id)` |
| `Extension<AdminClaims>` | 获取已通过 JWT 认证的当前管理员身份信息 | `claims.username`, `claims.role` |

---

## 3. 统一响应规范 (`ApiResponse<T>`)

为确保前端与第三方对接的标准化，Foundry 内置了强类型统一响应信封 `ApiResponse<T>`：

```json
{
  "code": 0,
  "message": "success",
  "data": { ... },
  "meta": null
}
```

### 常用辅助构建方法

```rust
use foundry::prelude::*;

// 1. 成功返回数据 (code=0, message="success")
ApiResponse::ok(data)
ApiResponse::success(data)

// 2. 携带自定义提示信息
ApiResponse::ok_msg(data, "操作成功已入库")

// 3. 资源创建成功 (自动设置 HTTP 201 Created)
ApiResponse::created(new_order)

// 4. 标准分页列表数据 (附带 page, limit, total, total_pages 等元数据)
ApiResponse::paginated(items, page, page_size, total_count)

// 5. 错误响应
ApiResponse::error("VALIDATION_FAILED")
```

> [!TIP]
> **自动实现 IntoResponse**：
> `ApiResponse<T>` 已经为任意满足 `T: Serialize` 的数据实现了 Axum 的 `IntoResponse`。在控制器中可以直接返回 `AppResult<ApiResponse<T>>` 与 `Ok(ApiResponse::ok(data))`，无需显式包裹 `Json(...)`，既清爽又高效。

---

## 4. 统一错误处理 (`AppError` 与 `AppResult`)

Foundry 使用类型安全的统一枚举 `AppError` 表达各类异常，并通过 Rust 的 `?` 运算符向上传播：

```rust
pub type AppResult<T> = Result<T, AppError>;
```

### 错误变体与 HTTP 状态码映射

| 错误变体 | 触发场景 | 映射 HTTP 状态码 |
|---|---|---|
| `AppError::Validation(msg)` | DTO 校验未通过 | `422 Unprocessable Entity` |
| `AppError::NotFound(msg)` | 目标实体/资源不存在 | `404 Not Found` |
| `AppError::Unauthorized(msg)` | 未登录或 JWT 令牌失效 | `401 Unauthorized` |
| `AppError::Forbidden(msg)` | 权限不足或无权访问当前子系统 | `403 Forbidden` |
| `AppError::Conflict(msg)` | 唯一索引冲突或业务状态不允许变更 | `409 Conflict` |
| `AppError::BadRequest(msg)` | 请求参数不合规或业务校验拦截 | `400 Bad Request` |
| `AppError::Database(msg)` | 底层数据库连接异常或 SQL 执行失败 | `500 Internal Server Error` |
| `AppError::Internal(msg)` | 未预期系统内部异常 | `500 Internal Server Error` |

### 控制器实战示例

```rust
// src/systems/shop/controllers/mod.rs
use foundry::prelude::*;
use crate::systems::shop::dto::{CreateOrderRequest, OrderResponse};
use crate::systems::shop::logic::OrderService;

pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 1. 验证入参格式 (如果校验失败，自动转换为 AppError::Validation 并返回 HTTP 422)
    payload.validate()?;

    // 2. 调度领域服务 (使用 ? 自动向上传播领域错误)
    let order = OrderService::create_order(&ctx, &state, payload).await?;

    // 3. 记录业务结构化日志
    info!(
        order_no = %order.order_no,
        amount = order.total_amount,
        "新订单创建成功"
    );

    // 4. 返回 HTTP 201 响应
    Ok(ApiResponse::created(order))
}
```

---

## 5. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 不要捕获错误后吞掉**：
> 在 Rust 中不要轻易使用 `.unwrap()` 或直接丢弃错误。推荐使用 `?` 向上传播，让统一错误中间件将其转化为标准的 JSON 错误信封返回给调用方。
> 
> **2. 提取器的顺序**：
> Axum 规定 `Json` 提取器会消费 HTTP 请求的 Body 数据流。因此如果有多个提取器，`Json(...)` 必须放在参数列表的末尾。

下一节：[04. 请求验证器与 DTO 设计](./04-validation/)

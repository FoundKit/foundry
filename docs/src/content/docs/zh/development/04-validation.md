---
title: 请求验证器与 DTO 设计
description: 掌握 Foundry 强类型参数验证器 (DTO) 设计、validator 宏注解、常用校验规则速查与 HTTP 422 错误截断机制。
---

# 请求验证器与 DTO 设计

本章内容：
1. 理解请求验证在前后端交互中的防御性编程价值；
2. 掌握基于 `validator` 宏的 DTO 数据传输对象设计；
3. 熟练运用 length, email, range, regex 等常用验证规则；
4. 学会处理复杂嵌套对象与数组列表的级联校验。

---

## 1. 验证器设计理念与防御性编程

Foundry 将数据验证直接与 Rust 的强类型系统深度绑定：
* **编译期强类型**：通过 `#[derive(Deserialize, Validate)]` 在 DTO 结构体字段上直接声明验证规则；
* **非侵入式拦截**：在控制器 Handler 顶部只需调用一行 `payload.validate()?`，若验证失败，框架会自动截断后续执行并返回结构化的 HTTP 422 校验失败响应；
* **清晰的数据边界**：DTO 隔离了外部网络请求结构与内部数据库实体模型，防止客户端恶意注入额外字段（Mass Assignment Vulnerability）。

---

## 2. 编写标准验证 DTO

我们在子系统的 `dto/` 目录（例如 `src/systems/shop/dto/mod.rs`）下统一定义请求和响应模型：

```rust
// src/systems/shop/dto/mod.rs
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 创建订单子项
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderItem {
    #[validate(range(min = 1, message = "商品 ID 必须为合法正整数"))]
    pub product_id: i64,

    #[validate(range(min = 1, max = 99, message = "单项购买数量必须在 1 到 99 之间"))]
    pub quantity: i32,
}

/// 创建订单请求 DTO
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 2, max = 32, message = "顾客姓名长度须为 2-32 字符"))]
    pub customer_name: String,

    #[validate(email(message = "必须提供合法的电子邮箱地址"))]
    pub customer_email: String,

    #[validate(length(min = 5, max = 255, message = "收货地址长度须在 5-255 字符之间"))]
    pub shipping_address: String,

    // 嵌套结构列表校验：必须使用 nested 标注级联校验子项
    #[validate(length(min = 1, message = "订单必须至少包含一件商品"))]
    #[validate(nested)]
    pub items: Vec<CreateOrderItem>,
}

/// 分页查询请求 DTO (只读查询通常使用 Option 可选字段)
#[derive(Debug, Clone, Deserialize)]
pub struct OrderListQuery {
    pub status: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 订单统一返回 DTO
#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub customer_email: String,
    pub total_amount: f64,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 3. 常用校验规则速查清单

| 规则名称 | 语法示例 | 说明 |
|---|---|---|
| **字符串长度** | `#[validate(length(min = 6, max = 32))]` | 限制字符长度（支持中英文） |
| **数值范围** | `#[validate(range(min = 0.01, max = 99999.0))]` | 限制整数或浮点数取值范围 |
| **电子邮箱** | `#[validate(email)]` | 校验是否符合标准邮箱格式 |
| **URL 链接** | `#[validate(url)]` | 校验是否为合法 HTTP/HTTPS 链接 |
| **正则表达式** | `#[validate(regex(path = *PHONE_REGEX))]` | 正则模式匹配（如手机号、身份证） |
| **必须包含** | `#[validate(contains = "@company.com")]` | 检查字符串子串是否存在 |
| **集合大小** | `#[validate(length(min = 1, max = 10))]` | 限制 `Vec<T>` 数组元素个数 |
| **嵌套级联** | `#[validate(nested)]` | 递归校验子结构体中的所有验证规则 |

---

## 4. 控制器中一键校验

在任何控制器中，验证入参极为清爽：

```rust
pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 1. 一行执行强校验：若非法自动截断并返回 HTTP 422
    payload.validate()?;

    // 2. 此时 payload 内部的所有数据均已保证符合业务约束
    let order = OrderService::create_order(&ctx, &state, payload).await?;
    Ok(ApiResponse::created(order))
}
```

当客户端传入非法数据（如邮箱缺失 `@` 或列表为空）时，客户端将立即收到标准化的错误返回：

```json
{
  "code": 422,
  "message": "Validation error: customer_email: 必须提供合法的电子邮箱地址",
  "data": null,
  "meta": null
}
```

---

## 5. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 嵌套数组必须加上 `#[validate(nested)]`**：
> 如果你的请求体中包含对象数组（如 `Vec<CreateOrderItem>`），仅在字段上声明 `length(min = 1)` 只能检查数组长度，**不会**自动触发每个子项的校验。必须额外添加 `#[validate(nested)]` 才会对数组中的每一项递归执行验证！
> 
> **2. 区分入参 DTO 与出参 DTO**：
> 不要把出参（如包含敏感密码哈希的内部模型）直接暴露给前端。坚持使用独立的 `Response` DTO 并只标记 `Serialize`，确保数据安全。

下一节：[05. 数据库配置与 Code-First ORM](./05-database-orm/)

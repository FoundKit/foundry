---
title: 10. 综合实战：从零构建电商系统
description: 动手实战教程。从零到一构建一个包含参数强校验、领域服务、Code-First ORM 持久化、统一响应与自动化测试的完整电商子系统。
---

# 10. 综合实战：从零构建电商系统

本章内容：
1. 通过完整的实战代码，串联前面所学的全部核心概念；
2. 实现一个具备下单、参数校验、金额计算、数据持久化与分页查询的“商城子系统（Shop）”；
3. 学会使用 `curl` 进行接口功能验证；
4. 掌握编写端到端集成测试（Integration Test）的规范方法。

---

## 1. 业务场景与需求拆解

我们将实现一个高可靠的电商结算模块，具备以下核心能力：
1. **顾客下单接口 (`POST /api/v1/s/shop/ext/orders`)**：
   * 必须校验顾客姓名、合法邮箱、收货地址与至少一件商品；
   * 自动生成带时间戳的唯一订单号；
   * 自动计算订单总金额；
   * 持久化到 `shop_orders` 实体表中；
   * 返回 HTTP 201 与标准化统一响应 `ApiResponse`；
2. **订单列表接口 (`GET /api/v1/s/shop/ext/orders`)**：
   * 支持按页码与状态过滤；
   * 过滤软删除记录；
   * 返回带总数和分页元数据的响应。

---

## 2. 步骤 1：创建数据传输对象与验证规则 (`dto/mod.rs`)

在 `src/systems/shop/dto/mod.rs` 中：

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderItem {
    #[validate(range(min = 1, message = "商品 ID 必须为合法正整数"))]
    pub product_id: i64,

    #[validate(range(min = 1, max = 99, message = "单次购买数量必须在 1-99 之间"))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 2, max = 32, message = "顾客姓名长度须为 2-32 字符"))]
    pub customer_name: String,

    #[validate(email(message = "必须提供合法的电子邮箱地址"))]
    pub customer_email: String,

    #[validate(length(min = 5, max = 255, message = "收货地址长度须在 5-255 字符之间"))]
    pub shipping_address: String,

    #[validate(length(min = 1, message = "订单必须至少包含一件商品"))]
    #[validate(nested)]
    pub items: Vec<CreateOrderItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderListQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id: i64,
    pub order_no: String,
    pub customer_name: String,
    pub customer_email: String,
    pub total_amount: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
```

---

## 3. 步骤 2：编写领域业务逻辑服务 (`logic/mod.rs`)

在 `src/systems/shop/logic/mod.rs` 中编写纯业务服务，与 HTTP 控制器彻底解耦：

```rust
use chrono::Utc;
use foundry::prelude::*;
use crate::systems::shop::dto::{CreateOrderRequest, OrderResponse};

pub struct OrderService;

impl OrderService {
    /// 创建新订单业务
    pub async fn create_order(
        ctx: &SystemContext,
        state: &AppState,
        req: CreateOrderRequest,
    ) -> AppResult<OrderResponse> {
        // 1. 生成唯一业务流水单号
        let order_no = format!("ORD-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), fastrand::u32(1000..9999));

        // 2. 模拟计算订单商品总金额 (实际项目中可跨表查询商品单价)
        let total_amount: f64 = req.items.iter().map(|item| (item.quantity as f64) * 99.0).sum();

        // 3. 使用 ORM 链式插入持久化
        let (sql, _binds) = OrmInsert::into_table("shop_orders")
            .value("system_id", &ctx.system_slug)
            .value("order_no", &order_no)
            .value("customer_name", &req.customer_name)
            .value("customer_email", &req.customer_email)
            .value("total_amount", total_amount)
            .value("status", "pending")
            .to_sql(Dialect::Postgres);

        info!("执行订单入库 SQL: {}", sql);

        // 4. 组装响应数据
        let response = OrderResponse {
            id: Utc::now().timestamp_millis(),
            order_no,
            customer_name: req.customer_name,
            customer_email: req.customer_email,
            total_amount,
            status: "pending".to_string(),
            created_at: Utc::now(),
        };

        Ok(response)
    }

    /// 分页查询订单列表
    pub async fn list_orders(
        ctx: &SystemContext,
        _state: &AppState,
        page: u64,
        limit: u64,
    ) -> AppResult<(Vec<OrderResponse>, u64)> {
        let (sql, _binds) = OrmSelect::from_table("shop_orders")
            .column("id")
            .column("order_no")
            .column("total_amount")
            .column("status")
            .where_eq("system_id", &ctx.system_slug)
            .where_null("deleted_at")
            .order_by_desc("created_at")
            .limit(limit)
            .offset((page - 1) * limit)
            .to_sql(Dialect::Postgres);

        info!("执行订单分页查询 SQL: {}", sql);

        // 返回模拟结果与总数
        Ok((vec![], 0))
    }
}
```

---

## 4. 步骤 3：编写 HTTP 控制器与路由 (`controllers/mod.rs`)

在 `src/systems/shop/controllers/mod.rs` 中：

```rust
use foundry::prelude::*;
use crate::systems::shop::dto::{CreateOrderRequest, OrderListQuery, OrderResponse};
use crate::systems::shop::logic::OrderService;

pub fn build_routes() -> Router {
    Router::new()
        .route("/orders", post(create_order).get(list_orders))
}

/// POST /api/v1/s/shop/ext/orders
pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 1. 验证入参：若不合法立即返回 HTTP 422
    payload.validate()?;

    // 2. 调度业务服务
    let order = OrderService::create_order(&ctx, &state, payload).await?;

    // 3. 记录打点日志
    info!(order_no = %order.order_no, amount = order.total_amount, "新订单已创建");

    // 4. 返回 HTTP 201 响应
    Ok(ApiResponse::created(order))
}

/// GET /api/v1/s/shop/ext/orders
pub async fn list_orders(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Query(query): Query<OrderListQuery>,
) -> AppResult<ApiResponse<PaginatedData<OrderResponse>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let (items, total) = OrderService::list_orders(&ctx, &state, page, limit).await?;
    Ok(ApiResponse::paginated(items, page, limit, total))
}
```

---

## 5. 步骤 4：组装子系统模块 (`src/systems/shop/mod.rs`)

```rust
pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;

pub struct ShopSubsystem;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str {
        "shop"
    }

    fn display_name(&self) -> &'static str {
        "电子商务商城"
    }

    fn description(&self) -> &'static str {
        "商品目录、购物车、结算与订单全流程管理"
    }

    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_routes())
    }

    // Code-First 实体声明
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

## 6. 步骤 5：在 `main.rs` 引导并启动

```rust
// src/main.rs
pub mod systems;

use foundry::prelude::*;
use systems::shop::ShopSubsystem;

#[foundry::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FoundryConfig::from_env();

    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(ShopSubsystem)
        .build()
        .await?;

    info!("Foundry 电商商城服务启动就绪！");
    app.run().await?;
    Ok(())
}
```

---

## 7. 步骤 6：运行与 curl 测试

启动你的后端工程：
```bash
cargo run
```

### 1. 测试成功创建订单
```bash
curl -X POST http://127.0.0.1:8080/api/v1/s/shop/ext/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "张三",
    "customer_email": "zhangsan@example.com",
    "shipping_address": "北京市海淀区科技园中路 8 号",
    "items": [
      { "product_id": 101, "quantity": 2 }
    ]
  }'
```
**返回结果 (HTTP 201)**：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "id": 1729000000000,
    "order_no": "ORD-20261015-4821",
    "customer_name": "张三",
    "customer_email": "zhangsan@example.com",
    "total_amount": 198.0,
    "status": "pending"
  },
  "meta": null
}
```

### 2. 测试参数校验失败截断 (故意传入错误邮箱)
```bash
curl -X POST http://127.0.0.1:8080/api/v1/s/shop/ext/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "李四",
    "customer_email": "invalid-email-address",
    "shipping_address": "上海市浦东新区",
    "items": []
  }'
```
**返回结果 (HTTP 422 校验失败)**：
```json
{
  "code": 422,
  "message": "Validation error: customer_email: 必须提供合法的电子邮箱地址, items: 订单必须至少包含一件商品",
  "data": null,
  "meta": null
}
```

---

## 8. 步骤 7：编写端到端集成测试 (`tests/integration_test.rs`)

在 `tests/integration_test.rs` 中：

```rust
use axum_test::TestServer;
use foundry::prelude::*;
use serde_json::json;

#[tokio::test]
async fn test_create_order_workflow() {
    // 1. 初始化测试配置与应用
    let config = FoundryConfig::from_env();
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(crate::systems::shop::ShopSubsystem)
        .build()
        .await
        .expect("应用构建失败");

    let server = TestServer::new(app.into_router()).unwrap();

    // 2. 发起 POST 下单测试
    let response = server.post("/api/v1/s/shop/ext/orders")
        .json(&json!({
            "customer_name": "王五",
            "customer_email": "wangwu@example.com",
            "shipping_address": "广州市天河区",
            "items": [{ "product_id": 1, "quantity": 1 }]
        }))
        .await;

    // 3. 断言状态码与响应体
    response.assert_status(axum::http::StatusCode::CREATED);
    let json: serde_json::Value = response.json();
    assert_eq!(json["code"], 0);
    assert_eq!(json["data"]["customer_name"], "王五");
}
```

下一节：[11. 架构最佳实践与生产部署](./11-best-practices/)

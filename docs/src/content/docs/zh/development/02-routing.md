---
title: 路由系统与子系统路由
description: 掌握 Foundry 统一命名空间路由体系、子系统扩展路由注册、HTTP 动词绑定与参数路由设计。
---

# 路由系统与子系统路由

本章内容：
1. 掌握 Foundry 平台全局路由空间规划；
2. 学会在子系统（`SubsystemModule`）中注册业务路由；
3. 理解扩展路由（`/ext/*`）与 Auto-CRUD 引擎路由的区别；
4. 熟练使用各种 HTTP 方法绑定与路由路径参数。

---

## 1. 全局统一路由空间规划

Foundry 在框架引擎层面规定了高度统一的全局路由命名空间。这种设计保证了多业务子系统共存时**永不发生路由命名冲突**：

```text
/ (根路径)
├── /health                               # 容器健康检查存活探针
├── /admin/*                              # 内置 React Admin 可视化管理后台
│
└── /api/v1/                              # 平台统一 API v1 空间
    ├── /admin/*                          # 平台与控制台管理 API (登录/子系统/审计/管理员)
    │
    └── /s/{subsystem}/                   # 业务子系统隔离空间 (如 /s/shop/ 或 /s/blog/)
        ├── /ext/*                        # 子系统自定义业务扩展 API (由 register_routes 挂载)
        │   ├── /orders                   # 例：POST /api/v1/s/shop/ext/orders
        │   └── /orders/{id}              # 例：GET  /api/v1/s/shop/ext/orders/123
        │
        └── /{model}                      # Auto-CRUD 引擎路由 (免写代码自动生成的 RESTful API)
            ├── GET    /api/v1/s/{subsystem}/{model}      (分页列表)
            ├── POST   /api/v1/s/{subsystem}/{model}      (创建实体)
            ├── GET    /api/v1/s/{subsystem}/{model}/{id} (实体详情)
            ├── PUT    /api/v1/s/{subsystem}/{model}/{id} (更新实体)
            └── DELETE /api/v1/s/{subsystem}/{model}/{id} (删除实体)
```

### 路由空间速查表

| 路由前缀 | 用途 | 编写方式 |
|---|---|---|
| `/health` | 服务探针 | 系统内置，返回 `OK` |
| `/api/v1/admin/*` | 平台运营控制台接口 | 系统内置（管理员账号管理、全局审计日志等） |
| `/api/v1/s/{subsystem}/ext/*` | **业务子系统自定义接口** | **开发者在 `register_routes` 中编写控制器并挂载** |
| `/api/v1/s/{subsystem}/{model}`| **Zero-DDL 动态模型接口** | **引擎自动按模型名生成，无需写任何代码** |
| `/admin/*` | 运营后台控制台 UI | 内置 React SPA 产物（或自定义静态单页） |

---

## 2. 子系统路由注册 (`register_routes`)

每个业务子系统都是一个实现了 `SubsystemModule` 的结构体。在 `src/systems/<slug>/mod.rs` 中重写 `register_routes` 方法即可挂载你的业务路由：

```rust
// src/systems/shop/mod.rs
pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;

pub struct ShopSubsystem;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str {
        "shop" // 子系统英文唯一标识
    }

    fn display_name(&self) -> &'static str {
        "电子商务商城"
    }

    fn description(&self) -> &'static str {
        "商品目录、购物车、结算与订单全流程管理"
    }

    // 挂载当前子系统的扩展路由
    // 最终访问路径为：/api/v1/s/shop/ext/*
    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_routes())
    }
}
```

> [!NOTE]
> **自动命名空间注入**：
> 在 `controllers::build_routes()` 中声明的相对路径（如 `/orders`），挂载后最终的完整访问 URL 自动为 `/api/v1/s/shop/ext/orders`。子系统开发者完全无需手动拼接前缀，框架保证了不同子系统之间的绝对隔离。

---

## 3. 路由定义与 HTTP 方法绑定

在 `src/systems/shop/controllers/mod.rs` 中，我们可以使用链式调用定义各种 HTTP 动词（GET, POST, PUT, DELETE, PATCH）的处理函数：

```rust
// src/systems/shop/controllers/mod.rs
use foundry::prelude::*;

pub fn build_routes() -> Router {
    Router::new()
        // 1. 同一路由绑定多种方法
        .route("/orders", post(create_order).get(list_orders))

        // 2. 路径参数路由 (使用 {param} 格式)
        .route("/orders/{id}", get(get_order_by_id).put(update_order))

        // 3. 业务动作路由
        .route("/orders/{id}/pay", post(pay_order))
        .route("/orders/{id}/cancel", post(cancel_order))

        // 4. 只读资源列表与搜索
        .route("/products", get(search_products))
}
```

### 控制器 Handler 签名规范

所有的控制器函数都是异步的（`async fn`），参数通过 Axum 提取器按需获取：

```rust
/// 创建订单 Handler
pub async fn create_order(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    // 业务处理...
    Ok(ApiResponse::ok(data))
}

/// 路径参数提取 Handler
pub async fn get_order_by_id(
    Extension(ctx): Extension<SystemContext>,
    Path(id): Path<i64>, // 自动将 URL 中的 {id} 解析为 i64
) -> AppResult<ApiResponse<OrderResponse>> {
    // 业务处理...
    Ok(ApiResponse::ok(data))
}
```

---

## 4. 复合参数路由提取

Axum 提供了极其灵活的路径参数解构能力：

### 提取单个参数
```rust
// 路由定义：.route("/orders/{id}", get(get_order))
async fn get_order(Path(id): Path<i64>) -> AppResult<ApiResponse<OrderResponse>> {
    // id 为 i64 类型
}
```

### 提取多个参数 (使用元组解构)
```rust
// 路由定义：.route("/categories/{cat_id}/products/{prod_id}", get(get_cat_product))
async fn get_cat_product(
    Path((cat_id, prod_id)): Path<(String, i64)>,
) -> AppResult<ApiResponse<ProductResponse>> {
    // 自动按顺序解构出 cat_id (String) 和 prod_id (i64)
}
```

---

## 5. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 路由路径语法**：
> Axum 0.8+ 采用 `{param}` 格式表示路径参数（如 `/orders/{id}`），不要写成老旧的 `:id` 格式。
> 
> **2. 路由冲突与匹配顺序**：
> 静态路由应优先定义于泛型通配路由之前。例如 `/orders/stats` 应该优先声明，避免被 `/orders/{id}` 误拦截。

下一节：[03. 控制器、参数与响应规范](./03-controllers/)

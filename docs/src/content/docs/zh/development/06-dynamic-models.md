---
title: Zero-DDL 动态数据模型与 Auto-CRUD
description: 掌握 Foundry 独创的 Zero-DDL 免建表动态数据模型、RecordStore 存储接口与即刻生成的 RESTful Auto-CRUD 引擎。
---

本章内容：
1. 理解 Zero-DDL 免模式（Schema-Free）动态数据模型的设计理念；
2. 掌握 `RecordStore` API 的增删改查使用方式；
3. 熟悉自动挂载的 RESTful Auto-CRUD HTTP 接口；
4. 清楚区分 Code-First ORM 与 Zero-DDL 动态模型的选型场景。

---

## 1. 为什么需要 Zero-DDL 动态模型？

在传统的数据库业务开发中，每新增一个业务实体，都必须经历：手写 SQL DDL -> 执行建表迁移 -> 编写模型类 -> 编写 CRUD 控制器 -> 编写前端管理表格。整个流程周期冗长。

Foundry 引入了 **Zero-DDL 动态数据模型引擎**：
* **免去 SQL DDL**：在可视化 Admin 控制台中直接点击创建数据模型，底层基于统一的 JSONB / 文档存储与动态倒排索引；
* **即刻生成 RESTful API**：模型一旦保存，框架引擎自动在 `/api/s/{system_slug}/{model_slug}` 路径暴露全套增删改查 REST 接口；
* **自动租户隔离**：所有动态记录在物理存储层自动打上 `system_slug` 标签，天然支持多系统安全隔离。

---

## 2. 选型对比：Code-First ORM vs Zero-DDL 动态模型

| 维度 | Code-First ORM 实体 (范式 A) | Zero-DDL 动态模型 (范式 B) |
|---|---|---|
| **定义方式** | 在 Rust 代码中通过 `EntitySchema` 声明 | 在 Admin 控制台可视化界面动态定义 |
| **底层存储** | 关系型独立物理表 (`shop_orders`) | 统一聚合存储表（Postgres JSONB / Mongo 文档） |
| **API 编写** | 自定义编写 Controller 与 Route | **自动生成，0 行代码即可对外提供 REST API** |
| **性能表现** | 极高（原生物理列与复合索引） | 很高（支持 GIN 倒排索引与 MongoDB Wildcard 索引） |
| **适用场景** | 核心交易、结算账单、高并发严谨业务 | **快速原型、运营配置、内容发布、工单收集、动态表单** |

---

## 3. 在代码中操作动态记录 (`RecordStore`)

除了通过 REST 接口，你也可以在子系统的领域服务中直接通过 `state.db` 读写动态记录：

```rust
use foundry::prelude::*;
use serde_json::json;

pub async fn handle_feedback(state: &AppState, ctx: &SystemContext) -> AppResult<()> {
    // 1. 创建动态记录 (参数依次为：子系统标识, 模型标识, JSON 数据)
    let record = state.db.records().create(
        &ctx.system_slug,   // 如 "shop"
        "feedbacks",        // 模型标识
        json!({
            "score": 5,
            "comment": "商品发货速度极快，包装完好！",
            "tags": ["品质优良", "物流好"]
        })
    ).await?;

    info!("动态记录创建成功: id={}", record.id);

    // 2. 根据 ID 查询详情
    let item = state.db.records().get_by_id(&ctx.system_slug, "feedbacks", record.id).await?;

    // 3. 分页列表查询
    let list_query = RecordQuery {
        page: Some(1),
        limit: Some(20),
        search: Some("发货".to_string()),
        ..Default::default()
    };
    let (items, total) = state.db.records().list(&ctx.system_slug, "feedbacks", list_query).await?;

    // 4. 更新动态记录
    state.db.records().update(
        &ctx.system_slug,
        "feedbacks",
        record.id,
        json!({
            "score": 5,
            "comment": "追加评价：客服也很耐心解答！"
        })
    ).await?;

    Ok(())
}
```

---

## 4. 自动生成的 RESTful Auto-CRUD 接口规范

一旦通过管理后台或 API 创建了模型（例如 `feedbacks`），框架将自动挂载以下 5 个端点：

| HTTP 方法 | 路径 | 描述 | 请求参数 / 请求体 |
|---|---|---|---|
| `GET` | `/api/s/{subsystem}/{model}` | 分页获取记录列表 | `?page=1&limit=20&search=关键词` |
| `POST` | `/api/s/{subsystem}/{model}` | 创建新记录 | 传入满足校验规则的 JSON 数据 |
| `GET` | `/api/s/{subsystem}/{model}/{id}` | 获取单条记录详情 | 路径中的数字 ID |
| `PUT` | `/api/s/{subsystem}/{model}/{id}` | 更新指定记录 | 传入需要更新的 JSON 字段 |
| `DELETE` | `/api/s/{subsystem}/{model}/{id}` | 删除指定记录 | 路径中的数字 ID |

### 响应数据格式示例

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "id": 1024,
    "system_slug": "shop",
    "model_slug": "feedbacks",
    "data": {
      "score": 5,
      "comment": "商品发货速度极快，包装完好！"
    },
    "created_at": "2026-10-15T09:30:00Z",
    "updated_at": "2026-10-15T09:30:00Z"
  },
  "meta": null
}
```

---

## 5. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 不要将千万级高频结算表建为 Zero-DDL 动态模型**：
> 动态模型非常适合敏捷开发、运营活动和配置数据。如果你的业务每天产生数百万级订单结算交易，推荐使用前文介绍的 [Code-First ORM 实体模型](./05-database-orm/)，充分发挥物理列与覆盖索引的极限吞吐。
> 
> **2. 动态模型同样受到生命周期钩子保护**：
> 对 Auto-CRUD 接口的写入与修改操作，同样会触发 `MutationHook`，你可以在钩子中对动态数据进行二次校验或风控拦截。

下一节：[07. 认证安全与分级 RBAC 权限控制](./07-auth-security/)

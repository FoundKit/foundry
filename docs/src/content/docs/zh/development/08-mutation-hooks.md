---
title: 08. 生命周期突变钩子 (Mutation Hook)
description: 掌握 Foundry 实体生命周期突变钩子流水线，实现数据前置校验拦截、字段动态注入与后置异步事件触发。
---

# 08. 生命周期突变钩子 (Mutation Hook)

> **本章目标**：
> 1. 理解生命周期突变钩子（Mutation Hook）的设计意图与运行机制；
> 2. 掌握 6 大核心生命周期事件方法的实现；
> 3. 学会在 `before_*` 钩子中进行数据校验拦截与动态字段注入；
> 4. 学会在 `after_*` 钩子中触发异步通知与缓存失效清理。

---

## 1. 钩子设计理念与拦截流水线

Foundry 提供了流水线式的 **生命周期突变钩子 (`MutationHook`)**：
* **前置拦截 (`before_*`)**：拥有可变数据引用（`&mut Value`），允许在数据落库前对其进行安全风控校验、补全默认字段或直接抛错阻断持久化；
* **后置通知 (`after_*`)**：在实体成功持久化到数据库后异步触发，通常用于记录审计流水、向消息队列投递事件、发送邮件或失效 Redis 缓存；
* **非侵入式装配**：钩子作为独立组件编写，在应用引导时通过 `.register_hook(...)` 插入流水线，与具体业务控制器代码解耦。

---

## 2. 6 大生命周期事件接口

通过实现 `foundry::prelude::MutationHook` 特征即可定义专属钩子：

```rust
use async_trait::async_trait;
use foundry::prelude::*;
use serde_json::Value;

pub struct ShopAuditHook;

#[async_trait]
impl MutationHook for ShopAuditHook {
    /// 1. 创建前：可拦截或向 data 动态注入字段
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if ctx.system_slug == "shop" && model_slug == "orders" {
            info!("即将为客户创建订单，开始风控审查: {:?}", data.get("customer_name"));

            // 示例：动态向持久化实体注入安全标记
            if let Some(obj) = data.as_object_mut() {
                obj.insert("risk_checked".to_string(), serde_json::json!(true));
            }
        }
        Ok(())
    }

    /// 2. 创建后：实体已成功写入数据库
    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        if ctx.system_slug == "shop" && model_slug == "orders" {
            info!("订单 #{} 持久化成功，准备触发消息通知与扣减库存", record_id);
        }
        Ok(())
    }

    /// 3. 更新前
    async fn before_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &mut Value,
    ) -> AppResult<()> {
        Ok(())
    }

    /// 4. 更新后
    async fn after_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        Ok(())
    }

    /// 5. 删除前
    async fn before_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }

    /// 6. 删除后
    async fn after_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }
}
```

---

## 3. 在应用启动时注册钩子 (`main.rs`)

钩子的注册极为简单，只需在 `FoundryApp::builder()` 流式调用中链式加入：

```rust
// src/main.rs
use foundry::prelude::*;
use systems::shop::{ShopSubsystem, ShopAuditHook};

#[foundry::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FoundryConfig::from_env();

    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(ShopSubsystem)
        .register_hook(ShopAuditHook) // 注册生命周期钩子
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

---

## 4. 初学者避坑指南 (Beginner Gotchas)

> [!WARNING]
> **1. 前置钩子抛出错误会熔断整个操作**：
> 如果在 `before_create` 或 `before_update` 中返回了 `Err(AppError::Validation(...))`，数据库持久化将被立即取消，并将该错误直接转换为 HTTP 响应返回给调用方。
> 
> **2. 必须做系统与模型匹配过滤**：
> `MutationHook` 是全局挂载的。如果不通过 `if ctx.system_slug == "..." && model_slug == "..."` 进行判断，该钩子会拦截所有子系统和所有模型的数据变动！

下一节：[09. 管理控制台与 postMessage 桥接](./09-admin-integration/)

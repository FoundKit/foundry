---
title: Lifecycle Mutation Hooks
description: Master Foundry's entity lifecycle mutation hook pipeline for pre-mutation validation, field injection, and post-mutation async side-effects.
---

# Lifecycle Mutation Hooks

This chapter covers:
1. Understand the design and purpose of `MutationHook`;
2. Implement the 6 lifecycle event methods;
3. Inject dynamic fields and intercept invalid mutations;
4. Trigger asynchronous side-effects and cache invalidation.

---

## 1. 6 Lifecycle Event Methods

Implement `MutationHook`:

```rust
use async_trait::async_trait;
use foundry::prelude::*;
use serde_json::Value;

pub struct ShopAuditHook;

#[async_trait]
impl MutationHook for ShopAuditHook {
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if ctx.system_slug == "shop" && model_slug == "orders" {
            // Dynamic enrichment
            if let Some(obj) = data.as_object_mut() {
                obj.insert("risk_checked".to_string(), serde_json::json!(true));
            }
        }
        Ok(())
    }

    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        if ctx.system_slug == "shop" && model_slug == "orders" {
            info!("Order #{} created. Trigger notification.", record_id);
        }
        Ok(())
    }

    async fn before_update(&self, _ctx: &SystemContext, _m: &str, _id: i64, _d: &mut Value) -> AppResult<()> { Ok(()) }
    async fn after_update(&self, _ctx: &SystemContext, _m: &str, _id: i64, _d: &Value) -> AppResult<()> { Ok(()) }
    async fn before_delete(&self, _ctx: &SystemContext, _m: &str, _id: i64) -> AppResult<()> { Ok(()) }
    async fn after_delete(&self, _ctx: &SystemContext, _m: &str, _id: i64) -> AppResult<()> { Ok(()) }
}
```

---

## 2. Hook Registration (`main.rs`)

```rust
let app = FoundryApp::builder()
    .config(config)
    .register_subsystem(ShopSubsystem)
    .register_hook(ShopAuditHook)
    .build()
    .await?;
```

Next chapter: [Admin Console & postMessage SDK](./09-admin-integration/)

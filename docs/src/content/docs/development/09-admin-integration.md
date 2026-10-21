---
title: Admin Console & postMessage SDK
description: Master Foundry's decoupled React Admin shell, subsystem custom page specifications, and postMessage token injection protocol.
---

This chapter covers:
1. Understand the decoupled Admin Console architecture (`apps/admin`);
2. Declare custom admin navigation items via `custom_admin_pages`;
3. Leverage automatic static file serving for sandboxed views;
4. Use the `postMessage` protocol for JWT authentication and theme synchronization.

---

## 1. Declaring Admin Menu Items

In `src/systems/shop/mod.rs`:

```rust
use foundry::prelude::*;

impl SubsystemModule for ShopSubsystem {
    fn slug(&self) -> &'static str { "shop" }
    fn display_name(&self) -> &'static str { "Shop" }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![
            CustomAdminPageSpec {
                key: "order_studio".to_string(),
                title: "Order Operations Studio".to_string(),
                icon: "ShoppingCart".to_string(),
                page_type: "iframe".to_string(),
                entry: "/admin/ext/shop/order_studio.html".to_string(),
                required_role: None,
            }
        ]
    }
}
```

> [!TIP]
> Place files in `src/systems/<slug>/custom_pages/` (e.g. `order_studio.html`). The engine serves them automatically under `/admin/ext/<slug>/*`.

---

## 2. postMessage Authentication Handshake

In `order_studio.html`:

```html
<script>
window.addEventListener('message', (event) => {
    const { type, token, user, system, theme } = event.data || {};
    if (type === 'FOUNDRY_AUTH_INIT') {
        console.log('Authenticated as:', user.username, 'in system:', system);
        // Use token in Authorization: Bearer header
        fetchOrders(token);
    }
});
</script>
```

Next chapter: [Hands-on Walkthrough: E-Commerce](./10-walkthrough-shop/)

---
title: Authentication & Hierarchical RBAC
description: Master Foundry's IAM security, Argon2id password hashing, JWT service, and 3-tier hierarchical RBAC subsystem tenant isolation.
---

This chapter covers:
1. Understand Foundry's IAM architecture;
2. Master Argon2id password hashing and JWT token verification;
3. Understand 3-tier role hierarchy (`super_admin`, `admin`, `topic_admin`);
4. Guard custom routes and enforce subsystem tenant isolation.

---

## 1. 3-Tier RBAC Architecture

* **`super_admin`**: Platform superuser with access across all subsystems.
* **`admin`**: General administrator subject to global platform policies.
* **`topic_admin`**: **Subsystem-scoped administrator** restricted to their assigned `allowed_systems` array (e.g. `["shop"]`).

---

## 2. Guarding Subsystem Routes

```rust
use foundry::prelude::*;

pub async fn admin_manage_order(
    Extension(claims): Extension<AdminClaims>,
    Extension(ctx): Extension<SystemContext>,
) -> AppResult<ApiResponse<()>> {
    // 1. Verify that the administrator has access to the current subsystem
    check_system_access(&claims, &ctx.system_slug)?;

    // 2. Optional: check for super_admin
    if claims.role != "super_admin" {
        return Err(AppError::Forbidden("Super admin required".into()));
    }

    Ok(ApiResponse::ok(()))
}
```

Next chapter: [Lifecycle Mutation Hooks](./08-mutation-hooks/)

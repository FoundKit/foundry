---
title: Zero-DDL Dynamic Models & Auto-CRUD
description: Master Foundry's Zero-DDL schema-free dynamic data models, RecordStore storage API, and instantly generated RESTful Auto-CRUD engine.
---

This chapter covers:
1. Understand Zero-DDL schema-free storage philosophy;
2. Use `RecordStore` APIs for creating, reading, updating, and querying records;
3. Understand auto-generated RESTful Auto-CRUD endpoints;
4. Choose between Code-First ORM and Zero-DDL Dynamic Models.

---

## 1. Zero-DDL Philosophy

Unlike traditional frameworks where every new table requires SQL DDL migrations, Foundry allows dynamic model definition via the Admin Console with instant REST API generation under `/api/s/{subsystem}/{model}`:

```rust
use foundry::prelude::*;
use serde_json::json;

// Create record in domain service
let record = state.db.records().create(
    &ctx.system_slug,
    "feedbacks",
    json!({
        "score": 5,
        "comment": "Fast shipping!"
    })
).await?;
```

---

## 2. Auto-CRUD Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/s/{subsystem}/{model}` | Paginated list with search |
| `POST` | `/api/s/{subsystem}/{model}` | Create record |
| `GET` | `/api/s/{subsystem}/{model}/{id}` | Get record |
| `PUT` | `/api/s/{subsystem}/{model}/{id}` | Update record |
| `DELETE` | `/api/s/{subsystem}/{model}/{id}` | Delete record |

Next chapter: [Authentication & Hierarchical RBAC](./07-auth-security/)

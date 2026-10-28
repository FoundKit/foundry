---
title: Zero-DDL Dynamic Models & Auto-CRUD
description: Master Foundry's Zero-DDL schema-free dynamic data models, RecordStore storage API, and instantly generated RESTful Auto-CRUD engine.
---

This chapter covers:
1. Understand Zero-DDL schema-free storage philosophy;
2. Storage structure clarification: row-level isolation per entity record;
3. Choose between Code-First ORM and Zero-DDL Dynamic Models (engine-specific trade-offs);
4. Use `RecordStore` APIs for creating, reading, updating, and querying records;
5. Understand auto-generated RESTful Auto-CRUD endpoints.

---

## 1. Zero-DDL Philosophy

Unlike traditional frameworks where every new table requires SQL DDL migrations, Foundry allows dynamic model definition via the Admin Console with instant REST API generation under `/api/s/{subsystem}/{model}`:

* **No SQL DDL required**: Create dynamic models instantly in the web Admin console;
* **Auto-generated REST APIs**: Five standard CRUD endpoints are mounted automatically;
* **Built-in multi-tenant isolation**: All records are tagged with `system_slug` in the storage layer.

### 1.1 Storage Architecture Clarification: One Row Per Entity Record

A common misconception among developers is: *“Does a dynamic model store all of its records inside a single giant JSON array in one row?”*

**No, absolutely not!**
* **A Model represents a logical table**: Its metadata (name, slug, permissions) is stored in the `models` table (taking 1 row).
* **Each individual business record (e.g. an article, an order feedback)**: Occupies **an independent row** in `model_records` (or an independent BSON Document in MongoDB).
* Each row has its own auto-increment `id`, `created_at`, `deleted_at`, and its custom attributes stored in the `data` (JSON) column.

This row-level architecture eliminates "Unbounded Array" anti-patterns, preserves document size limits (like MongoDB's 16MB cap), and enables true row-level write concurrency.

---

## 2. Decision Matrix: Code-First ORM vs Zero-DDL Dynamic Models

| Dimension | Code-First ORM Entity (Paradigm A) | Zero-DDL Dynamic Model (Paradigm B) |
|---|---|---|
| **Definition** | Declared in Rust code via `EntitySchema` | Defined dynamically in web Admin UI |
| **Storage** | Dedicated relational table (`shop_orders`) | Unified storage table (`model_records`) |
| **API** | Custom Controller & Router handlers | **Zero lines of code; auto-mounted Auto-CRUD** |
| **Performance** | Highest (Native physical columns & indexes) | High (PostgreSQL GIN / MongoDB Wildcard index) |
| **Use Cases** | Core billing, orders, high-concurrency ledger | **Surveys, form submissions, CMS, dynamic configs** |

### 2.1 Database Engine Trade-offs (Why MySQL Benefits from Dedicated Tables)

1. **PostgreSQL**: Features native `JSONB` with `GIN(data)` inverted indexes. Even in the unified table, queries filtering inside JSON attributes hit GIN indexes in milliseconds.
2. **MongoDB**: Native BSON document engine with Wildcard index (`data.$**`), naturally optimized for dynamic schema queries.
3. **MySQL / MariaDB**:
   * **List pagination is fast**: Covering index `(model_id, deleted_at, created_at DESC)` and deferred joins make standard paginated listing very fast.
   * **Deep JSON property filtering penalty**: Lacking native inverted indexes across arbitrary JSON keys, complex filtering inside JSON properties requires row deserialization scans.
   * **Conclusion**: On MySQL/MariaDB, use Zero-DDL models for agile forms and configurations; for high-frequency transactional data (e.g. orders, inventory), strongly prefer [Code-First ORM dedicated physical tables](./05-database-orm/).

---

## 3. Operating Dynamic Records in Code (`RecordStore`)

```rust
use foundry::prelude::*;
use serde_json::json;

pub async fn handle_feedback(state: &AppState, ctx: &SystemContext) -> AppResult<()> {
    // 1. Create dynamic record
    let record = state.db.records().create(
        &ctx.system_slug,
        "feedbacks",
        json!({
            "score": 5,
            "comment": "Fast shipping!",
            "tags": ["quality", "logistics"]
        })
    ).await?;

    // 2. Fetch by ID
    let item = state.db.records().get_by_id(&ctx.system_slug, "feedbacks", record.id).await?;

    // 3. Paginated list
    let query = RecordQuery {
        page: Some(1),
        page_size: Some(20),
        search: Some("shipping".to_string()),
        ..Default::default()
    };
    let (items, total) = state.db.records().list(&ctx.system_slug, "feedbacks", query).await?;

    // 4. Update dynamic record
    state.db.records().update(
        &ctx.system_slug,
        "feedbacks",
        record.id,
        json!({ "score": 5, "comment": "Updated: Great service!" })
    ).await?;

    Ok(())
}
```

---

## 4. Auto-CRUD Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/s/{subsystem}/{model}` | Paginated list with search |
| `POST` | `/api/s/{subsystem}/{model}` | Create record |
| `GET` | `/api/s/{subsystem}/{model}/{id}` | Get single record |
| `PUT` | `/api/s/{subsystem}/{model}/{id}` | Update record |
| `DELETE` | `/api/s/{subsystem}/{model}/{id}` | Soft-delete record |

---

## 5. Model & Field Schema Lifecycle Management

### 5.1 Model Renaming & Modification
* Modify model names, descriptions, and slugs dynamically.
* **Automatic Record Migration**: When changing slug (e.g. `tool` -> `tools`), the engine automatically updates the `model_slug` of all existing records in the database with zero downtime.

### 5.2 Soft-Delete & Trash Bin Restoration
* Deleting a model marks `deleted_at = NOW()`, preserving all underlying records safely.
* Deleted models are archived in the **Trash Bin**, where administrators can restore them with automatic conflict detection.

### 5.3 Field CRUD & Smooth Data Evolution
* Add, edit, or soft-delete fields at any time.
* **Preserved Raw Data**: Unchanged records retain all historical JSON attributes in storage.
* **Adaptive Display**: Tables only show current active fields.
* **Automatic Field Purging**: Re-saving or updating existing records automatically purges obsolete keys (such as removed fields `name` or `des`), keeping data tidy and conformant.

### 5.4 Relation Field Type
* Reference foreign records across models using `relation` type fields.
* Specify `target_model` in field options. Visual Data Explorer displays intuitive ID-binding dropdown pickers and relation badges.

---

Next chapter: [Authentication & Hierarchical RBAC](./07-auth-security/)

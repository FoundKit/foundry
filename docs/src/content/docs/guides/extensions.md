---
title: Subsystems & Custom Features Guide
description: Learn how to develop modular business subsystems, custom endpoints, database integration, Admin UI extensions, and mutation hooks with Foundry.
---

# Subsystems & Custom Features Guide

Foundry empowers developers to build modular, maintainable backend applications by organizing business capabilities into **Subsystems** and **Lifecycle Hooks**.

---

## 1. Subsystem Architecture

A Subsystem is a self-contained business domain living inside your application (e.g. `src/systems/blog/` or `src/systems/billing/`).

```text
src/systems/blog/
├── mod.rs               # SubsystemModule trait implementation
├── controllers/         # Axum HTTP handlers mounted at /api/v1/s/blog/ext/*
│   └── mod.rs
├── logic/               # Pure domain services with database access
│   └── mod.rs
├── dto/                 # Request/response structs and validator rules
│   └── mod.rs
└── custom_pages/        # Custom Admin UI views (HTML, React, Vue)
    └── article_editor.html
```

---

## 2. Implementing a Subsystem (Three-Layer Architecture)

### Step 1: Define DTOs & Validation (`dto/mod.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateArticleRequest {
    #[validate(length(min = 3, max = 120, message = "Title must be 3-120 characters"))]
    pub title: String,
    #[validate(length(min = 10, message = "Content must be at least 10 characters"))]
    pub content: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArticleResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author: String,
}
```

### Step 2: Implement Domain Business Logic with Database Access (`logic/mod.rs`)

```rust
use crate::systems::blog::dto::{CreateArticleRequest, ArticleResponse};
use foundry::prelude::*;

pub struct ArticleService;

impl ArticleService {
    pub async fn create_article(
        ctx: &SystemContext,
        db: &DbPool,
        req: CreateArticleRequest,
    ) -> AppResult<ArticleResponse> {
        // Execute database query or business transactions
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO articles (system_slug, title, content, author, created_at)
             VALUES ($1, $2, $3, $4, NOW())
             RETURNING id"
        )
        .bind(&ctx.system_slug)
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.author)
        .fetch_one(db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ArticleResponse {
            id: row.0,
            title: req.title,
            content: req.content,
            author: req.author,
        })
    }
}
```

### Step 3: Implement HTTP Controller (`controllers/mod.rs`)

```rust
use axum::{extract::Extension, routing::post, Json, Router};
use foundry::prelude::*;
use validator::Validate;
use crate::systems::blog::dto::{CreateArticleRequest, ArticleResponse};
use crate::systems::blog::logic::ArticleService;

pub fn build_routes() -> Router {
    Router::new().route("/articles", post(handle_create_article))
}

pub async fn handle_create_article(
    Extension(ctx): Extension<SystemContext>,
    Extension(db): Extension<DbPool>,
    Json(payload): Json<CreateArticleRequest>,
) -> AppResult<Json<ApiResponse<ArticleResponse>>> {
    payload.validate()?;
    let article = ArticleService::create_article(&ctx, &db, payload).await?;
    Ok(Json(ApiResponse::success(article)))
}
```

### Step 4: Implement `SubsystemModule` (`mod.rs`)

```rust
pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub struct BlogSubsystem;

impl SubsystemModule for BlogSubsystem {
    fn slug(&self) -> &'static str {
        "blog"
    }

    fn display_name(&self) -> &'static str {
        "Blog & Content Publishing"
    }

    fn description(&self) -> &'static str {
        "Article authoring, tag categorization, and publishing pipeline"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());

        // Serve custom Admin UI pages under /custom-pages
        let custom_dir = PathBuf::from("src/systems/blog/custom_pages");
        if custom_dir.exists() {
            r = r.nest_service("/custom-pages", ServeDir::new(custom_dir));
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "article_editor".to_string(),
            title: "Article Studio".to_string(),
            icon: "FileEdit".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/blog/ext/custom-pages/article_editor.html".to_string(),
            required_role: None, // e.g. Some("super_admin".to_string())
        }]
    }
}
```

---

## 3. Registering Subsystems in `main.rs`

In your application's entry point:

```rust
use foundry::prelude::*;
use systems::BlogSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = FoundryApp::builder()
        .register_subsystem(BlogSubsystem)
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

---

## 4. Custom Admin UI Pages & SDK Bridge

Custom subsystem pages embedded in the Foundry Admin Shell automatically receive authentication context and theme events via `window.addEventListener('message', ...)`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Article Studio</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 min-h-screen p-6">
  <div class="max-w-4xl mx-auto space-y-4">
    <h1 class="text-xl font-bold text-indigo-600 dark:text-indigo-400">📝 Article Studio</h1>
    <p id="admin-badge" class="text-xs text-slate-500">Connecting to Foundry Admin Shell...</p>
  </div>
  <script>
    let authContext = null;

    // 1. Receive context from Foundry Admin Shell
    window.addEventListener('message', function(event) {
      if (event.data?.type === 'FOUNDRY_INIT') {
        authContext = event.data.payload;
        if (authContext.theme === 'dark') document.documentElement.classList.add('dark');
        document.getElementById('admin-badge').innerText =
          `Authenticated as: ${authContext.admin.username} (${authContext.admin.role}) for subsystem: ${authContext.subsystemSlug}`;
      }
    });

    // 2. Send toast notifications back to parent Admin Shell
    function showToast(message, level = 'success') {
      window.parent.postMessage({
        type: 'FOUNDRY_TOAST',
        payload: { message, level }
      }, '*');
    }
  </script>
</body>
</html>
```

---

## 5. Lifecycle Mutation Hooks

Intercept dynamic model mutations through the `MutationHook` trait:

```rust
use foundry::prelude::*;
use serde_json::Value;

pub struct AuditHook;

#[async_trait]
impl MutationHook for AuditHook {
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        tracing::info!("Creating record in model '{}' (system: {})", model_slug, ctx.system_slug);
        Ok(())
    }

    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        tracing::info!("Created record {} in '{}'", record_id, model_slug);
        Ok(())
    }
}
```

Register hooks in your builder:

```rust
let app = FoundryApp::builder()
    .register_hook(AuditHook)
    .build()
    .await?;
```

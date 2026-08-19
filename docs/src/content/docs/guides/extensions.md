---
title: Subsystems & Extensions
description: Guide to extending Foundry with custom backend APIs, Admin UI pages, and standalone repositories.
---

# Subsystem Extensions & Custom Admin UI Guide

Foundry provides a code-first extension engine allowing sub-systems to define **custom backend APIs**, **custom Admin UI pages**, and **standalone decoupled repository hosting**.

---

## 1. Custom Backend Extension APIs (`/api/v1/s/{slug}/ext/*`)

Custom controllers written in Rust (or dynamic WASM modules) are mounted under the dedicated extension path prefix `/:system_slug/ext/*` to eliminate route collisions with dynamic Auto-CRUD models.

### Example: Writing a Custom Controller in Rust

#### 1. Define Request/Response DTO (`dto/draw_dto.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DrawRequest {
    #[validate(length(min = 1))]
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct DrawResponse {
    pub prize_name: String,
    pub is_win: bool,
}
```

#### 2. Implement Domain Logic Service (`logic/draw_service.rs`)

```rust
use crate::carnival_demo::dto::{DrawRequest, DrawResponse};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;

pub struct DrawService;

impl DrawService {
    pub async fn execute(ctx: &SystemContext, req: DrawRequest) -> AppResult<DrawResponse> {
        let configs = ctx.configs();
        Ok(DrawResponse {
            prize_name: "Grand Prize".to_string(),
            is_win: true,
        })
    }
}
```

#### 3. Implement HTTP Handler (`controllers/draw_controller.rs`)

```rust
use axum::{extract::Extension, Json};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::ApiResponse;
use validator::Validate;
use crate::carnival_demo::dto::{DrawRequest, DrawResponse};
use crate::carnival_demo::logic::DrawService;

pub async fn handle_draw(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<DrawRequest>,
) -> AppResult<Json<ApiResponse<DrawResponse>>> {
    payload.validate()?;
    let result = DrawService::execute(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

#### 4. Register Extension Routes (`controllers/mod.rs`)

```rust
use axum::{routing::post, Router};

pub fn build_routes() -> Router {
    Router::new().route("/draw", post(draw_controller::handle_draw))
}
```

Endpoint path: `POST /api/v1/s/carnival_demo/ext/draw`

---

## 2. Custom Subsystem Admin UI Pages (`CustomAdminPageSpec`)

Subsystems can register custom admin UI views (e.g. interactive dashboards, custom operational tools) that seamlessly integrate into the Foundry Admin UI shell with automatic JWT token and theme injection.

### Specification Schema (`CustomAdminPageSpec`)

Custom pages can be declared in Rust (`SubsystemModule::custom_admin_pages()`) or in `subsystem.json`:

```json
{
  "slug": "vip_mall",
  "display_name": "VIP Mall Subsystem",
  "custom_pages": [
    {
      "key": "vip_overview",
      "title": "VIP Analytics Dashboard",
      "icon": "Crown",
      "type": "iframe",
      "entry": "/api/v1/s/vip_mall/ext/custom-pages/overview.html"
    }
  ]
}
```

### Embedded View SDK Bridge (`window.FoundryBridge`)

Embedded custom admin pages receive initialization messages from the Foundry Admin UI shell:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>VIP Overview</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-100">
  <h1 class="text-xl font-bold">Custom Subsystem View</h1>
  <p id="token-display" class="text-xs font-mono text-slate-500">Checking auth token...</p>

  <script>
    // Listen for initialization payload from Foundry Admin Shell
    window.addEventListener('message', function(event) {
      if (event.data && event.data.type === 'FOUNDRY_INIT') {
        const { token, subsystemSlug, admin, theme } = event.data.payload;
        document.getElementById('token-display').innerText = 'Token: ' + token.substring(0, 20) + '...';

        if (theme === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
    });

    // Invoke core platform APIs
    async function loadConfigs() {
      const res = await fetch('/api/v1/s/vip_mall/configs');
      const data = await res.json();
      console.log(data);
    }
  </script>
</body>
</html>
```

---

## 3. Standalone External Subsystem Hosting

To keep core platform updates clean, subsystems can be hosted in separate Git repositories.

1. **Scaffold external subsystem directory**:
   ```bash
   cargo run --bin foundry-cli -- system new-external vip_mall --name "VIP Mall Standalone"
   ```
2. **Directory Structure**:
   ```
   external_systems/vip_mall/
   ├── subsystem.json
   └── custom_pages/
       └── overview.html
   ```
3. **Dynamic Discovery**:
   Foundry automatically scans `FOUNDRY_SYSTEMS_DIR` environment variable and `./external_systems` at startup.

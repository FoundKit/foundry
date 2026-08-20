---
title: Subsystems & Custom Repository Guide
description: Guide to extending Foundry with custom backend APIs, Admin UI pages, and a single unified custom code repository.
---

# Subsystems Extension & Unified Custom Repository Guide

Foundry provides a **"Core Platform Base + Single Custom Systems Repository"** decoupled extension architecture. All custom subsystems (custom APIs + domain logic + custom Admin UI pages + manifests) for your team or organization reside in **one single Git repository**, with each subsystem living in its own self-contained directory.

---

## 1. Core Architecture: Why a Single Custom Repository?

```
[Upstream Base Infra: foundry] ───(Git Submodule / Mount)───> [User Custom Repo: foundry-systems]
               │                                                              │
    (git pull upstream main)                                       (Contains ALL custom subsystems)
    100% Conflict-Free Upgrades!                                   • carnival_demo/
               │                                                   • vip_mall/
               │                                                   • order_center/
               │                                                   • payment_gateway/
               │                                                              │
               └───────────────────> Unified Packaging Pipeline <─────────────┘
                                               │
                                   (Compiled Production Bundle)
```

- **Minimal Maintenance**: You only maintain **one** custom repository for all your business subsystems.
- **0-Conflict Upgrades**: All custom code is 100% physically decoupled from the `foundry` base. When Foundry releases updates or security fixes, pull upstream without touching your custom code!
- **Unified Release Packaging**: At build time, the packaging pipeline merges the base engine and your single custom repository into a single native binary or container.

---

## 2. Self-Contained Subsystem Directory Standard

Within your custom repository, each subsystem directory is self-contained:

```
foundry-systems/                     # Single Custom Systems Git Repository
├── carnival_demo/                   # Subsystem A (Self-contained directory)
│   ├── mod.rs                       # [Entry] SubsystemModule trait implementation
│   ├── subsystem.json               # [Manifest] Subsystem metadata & custom page specs
│   ├── controllers/                 # [Presentation] Axum HTTP controllers (/api/v1/s/:slug/ext/*)
│   │   ├── mod.rs                   # Route export
│   │   └── draw_controller.rs       # Request extraction & handler
│   ├── logic/                       # [Domain] Pure domain services (ctx.model / ctx.configs)
│   │   ├── mod.rs
│   │   └── draw_service.rs
│   ├── dto/                         # [Contract] DTOs & validator constraints
│   │   ├── mod.rs
│   │   └── draw_dto.rs
│   └── custom_pages/                # [View] Custom Admin UI views (HTML/React)
│       ├── lottery_dashboard.html
│       └── wheel_control.html
├── vip_mall/                        # Subsystem B
│   └── ...
└── order_center/                    # Subsystem C
    └── ...
```

---

## 3. Writing Custom Business Endpoints (Three-Layer Pattern)

### Step 1: Define DTOs & Validation (`dto/draw_dto.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DrawRequest {
    #[validate(length(min = 1, message = "User ID cannot be empty"))]
    pub user_id: String,
    #[validate(range(min = 1, max = 100, message = "Lucky number must be 1-100"))]
    pub lucky_number: u32,
}

#[derive(Debug, Serialize)]
pub struct DrawResponse {
    pub prize_name: String,
    pub is_winner: bool,
}
```

### Step 2: Implement Domain Business Logic (`logic/draw_service.rs`)

```rust
use crate::carnival_demo::dto::{DrawRequest, DrawResponse};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;

pub struct DrawService;

impl DrawService {
    pub async fn execute(ctx: &SystemContext, req: DrawRequest) -> AppResult<DrawResponse> {
        let configs = ctx.configs();
        let is_winner = req.lucky_number == 14;
        let prize_name = if is_winner { "Grand Prize" } else { "Thanks for participating" };

        Ok(DrawResponse {
            prize_name: prize_name.to_string(),
            is_winner,
        })
    }
}
```

### Step 3: Implement HTTP Controller (`controllers/draw_controller.rs`)

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

### Step 4: Mount Routes and Static Pages (`mod.rs`)

```rust
pub mod controllers;
pub mod dto;
pub mod logic;

use std::path::PathBuf;
use axum::Router;
use foundry_core::SubsystemModule;
use tower_http::services::ServeDir;

pub struct CarnivalDemoModule;

impl SubsystemModule for CarnivalDemoModule {
    fn slug(&self) -> &'static str {
        "carnival_demo"
    }

    fn display_name(&self) -> &'static str {
        "Carnival 2026 Demo Subsystem"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());
        let possible_dirs = [
            PathBuf::from("systems/src/carnival_demo/custom_pages"),
            PathBuf::from("static/custom_pages/carnival_demo"),
        ];
        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<foundry_core::CustomAdminPageSpec> {
        vec![
            foundry_core::CustomAdminPageSpec {
                key: "lottery_dashboard".to_string(),
                title: "Lottery Dashboard".to_string(),
                icon: "Gift".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/carnival_demo/ext/custom-pages/lottery_dashboard.html".to_string(),
                required_role: None,
            },
        ]
    }
}
```

---

## 4. Custom Admin UI Pages & SDK Bridge

Custom subsystem admin views are placed in `custom_pages/` and automatically embedded into the Foundry Admin UI shell:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Lottery Dashboard</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold">🎁 Custom Subsystem Console</h1>
  <p id="admin-info" class="text-xs text-slate-500 mt-2">Connecting...</p>

  <script>
    let authContext = null;

    // 1. Receive initialization context from Foundry Admin shell
    window.addEventListener('message', function(event) {
      if (event.data?.type === 'FOUNDRY_INIT') {
        authContext = event.data.payload;
        document.getElementById('admin-info').innerText =
          `Active Admin: ${authContext.admin.username} (${authContext.admin.role})`;

        // Auto dark mode sync
        if (authContext.theme === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
      }
    });

    // 2. Send toast notification to Admin UI shell
    function notifyParent(msg) {
      window.parent.postMessage({
        type: 'FOUNDRY_TOAST',
        payload: { message: msg, level: 'success' }
      }, '*');
    }
  </script>
</body>
</html>
```

---

## 5. Team Custom Systems Repository Workflow

### Step 1: Initialize Single Custom Subsystems Repository

```bash
cargo run --bin foundry-cli -- system init-repo ../my-foundry-systems
```

### Step 2: Validate All Subsystems in Custom Repository

```bash
cargo run --bin foundry-cli -- system validate ../my-foundry-systems
```

---

## 6. Unified Production Packaging

Merge base infrastructure and your single custom systems repo into a release bundle:

```bash
./scripts/build-release.sh --systems-dir ../my-foundry-systems
```


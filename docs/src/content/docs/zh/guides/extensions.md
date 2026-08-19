---
title: 子系统与扩展开发
description: 使用自定义后端 API、管理后台页面与独立仓库扩展 Foundry。
---

# 子系统扩展与自定义管理后台指南

Foundry 提供了 Code-first 优先的扩展引擎，允许子系统定义 **自定义后端 API 接口**、**自定义管理后台页面** 以及 **解耦的独立代码仓库托管**。

---

## 1. 自定义后端扩展 API (`/api/v1/s/{slug}/ext/*`)

使用 Rust 编写的自定义控制器（或动态 WASM 模块）挂载在专用的扩展路径前缀 `/:system_slug/ext/*` 下，避免与动态 Auto-CRUD 模型产生路由冲突。

### 示例：使用 Rust 编写自定义控制器

#### 1. 定义请求/响应 DTO (`dto/draw_dto.rs`)

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

#### 2. 实现领域业务逻辑服务 (`logic/draw_service.rs`)

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

#### 3. 实现 HTTP 处理函数 (`controllers/draw_controller.rs`)

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

#### 4. 注册扩展路由 (`controllers/mod.rs`)

```rust
use axum::{routing::post, Router};

pub fn build_routes() -> Router {
    Router::new().route("/draw", post(draw_controller::handle_draw))
}
```

接口请求路径: `POST /api/v1/s/carnival_demo/ext/draw`

---

## 2. 自定义子系统管理后台页面 (`CustomAdminPageSpec`)

子系统可以注册自定义的管理后台页面（例如可视化仪表盘、运营工具），自动注入 JWT Token 和当前主题色，无缝嵌入 Foundry Admin UI 壳层中。

### 规范声明 (`CustomAdminPageSpec`)

可以在 Rust 代码中声明（`SubsystemModule::custom_admin_pages()`）或在 `subsystem.json` 中配置：

```json
{
  "slug": "vip_mall",
  "display_name": "VIP Mall Subsystem",
  "custom_pages": [
    {
      "key": "vip_overview",
      "title": "VIP 数据看板",
      "icon": "Crown",
      "type": "iframe",
      "entry": "/api/v1/s/vip_mall/ext/custom-pages/overview.html"
    }
  ]
}
```

### 嵌入式页面 SDK 桥接 (`window.FoundryBridge`)

嵌入的自定义管理页面接收来自 Foundry Admin UI 外壳的初始化消息：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>VIP 概览</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-white dark:bg-slate-900 text-slate-800 dark:text-slate-100">
  <h1 class="text-xl font-bold">自定义子系统视图</h1>
  <p id="token-display" class="text-xs font-mono text-slate-500">正在检查认证 Token...</p>

  <script>
    // 监听来自 Foundry Admin 外壳的初始化事件
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

    // 调用平台核心 API
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

## 3. 独立外部子系统仓库托管

为了保持核心平台的独立性与整洁，子系统可以托管在独立的 Git 仓库中。

1. **创建外部子系统脚手架**:
   ```bash
   cargo run --bin foundry-cli -- system new-external vip_mall --name "VIP Mall Standalone"
   ```
2. **目录结构**:
   ```
   external_systems/vip_mall/
   ├── subsystem.json
   └── custom_pages/
       └── overview.html
   ```
3. **动态发现与加载**:
   Foundry 服务在启动时会自动扫描环境变量 `FOUNDRY_SYSTEMS_DIR` 以及 `./external_systems` 目录。

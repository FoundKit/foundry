---
title: 子系统与自定义功能开发
description: 学习如何使用 Foundry 开发模块化业务子系统、数据库业务操作、自定义 API 控制器、Admin 扩展大屏与生命周期钩子。
---

# 子系统与自定义功能开发指南

Foundry 提倡高内聚、低耦合的模块化设计。业务开发者可以将独立业务组织为 **子系统 (Subsystem)** 与 **生命周期钩子 (Mutation Hook)**。

---

## 1. 业务子系统目录结构

在用户业务工程中，子系统通常放置在 `src/systems/<slug>/`：

```text
src/systems/blog/
├── mod.rs               # SubsystemModule 特征实现入口
├── controllers/         # Axum HTTP 控制器 (挂载至 /api/v1/s/blog/ext/*)
│   └── mod.rs
├── logic/               # 纯业务领域服务 (包含数据库读写)
│   └── mod.rs
├── dto/                 # 请求与响应 DTO 结构体及 validator 校验注解
│   └── mod.rs
└── custom_pages/        # 自定义 Admin 运营看板 (HTML, React, Vue)
    └── article_editor.html
```

---

## 2. 编写子系统标准步骤 (三层架构模式)

### 步骤 1: 定义 DTO 与参数校验 (`dto/mod.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateArticleRequest {
    #[validate(length(min = 3, max = 120, message = "标题长度须在 3-120 字符之间"))]
    pub title: String,
    #[validate(length(min = 10, message = "正文至少需要 10 个字符"))]
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

### 步骤 2: 编写领域服务与数据库交互 (`logic/mod.rs`)

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
        // 执行原生数据库写入或复杂事务
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

### 步骤 3: 编写 HTTP 控制器并提取 DbPool (`controllers/mod.rs`)

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

### 步骤 4: 实现 `SubsystemModule` 特征 (`mod.rs`)

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
        "博客与内容管理系统"
    }

    fn description(&self) -> &'static str {
        "文章内容创作、标签分类与发布工作流"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());

        // 挂载自定义 Admin 静态页面
        let custom_dir = PathBuf::from("src/systems/blog/custom_pages");
        if custom_dir.exists() {
            r = r.nest_service("/custom-pages", ServeDir::new(custom_dir));
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "article_editor".to_string(),
            title: "文章创作工作室".to_string(),
            icon: "FileEdit".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/blog/ext/custom-pages/article_editor.html".to_string(),
            required_role: None, // 如需限制超管可见：Some("super_admin".to_string())
        }]
    }
}
```

---

## 3. 在应用启动器中注册

在用户应用 `main.rs` 中：

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

## 4. 自定义 Admin 运营看板与 SDK Bridge 通信

嵌入在 Foundry Admin 外壳中的页面会自动通过 `postMessage` 接收管理员登录凭据和主题：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>文章创作工作室</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 min-h-screen p-6">
  <div class="max-w-4xl mx-auto space-y-4">
    <h1 class="text-xl font-bold text-indigo-600 dark:text-indigo-400">📝 文章创作工作室</h1>
    <p id="admin-badge" class="text-xs text-slate-500">正在连接 Foundry Admin 外壳...</p>
  </div>
  <script>
    let authContext = null;

    // 1. 监听来自 Admin Shell 的初始化广播
    window.addEventListener('message', function(event) {
      if (event.data?.type === 'FOUNDRY_INIT') {
        authContext = event.data.payload;
        if (authContext.theme === 'dark') document.documentElement.classList.add('dark');
        document.getElementById('admin-badge').innerText =
          `当前登录: ${authContext.admin.username} (${authContext.admin.role}) | 子系统: ${authContext.subsystemSlug}`;
      }
    });

    // 2. 向外壳发送 Toast 通知
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

## 5. 生命周期变更钩子 (Mutation Hooks)

通过实现 `MutationHook` 特征拦截动态数据模型的写操作：

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
        tracing::info!("在子系统 '{}' 创建模型 '{}' 记录", ctx.system_slug, model_slug);
        Ok(())
    }

    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        tracing::info!("记录 {} 创建完成", record_id);
        Ok(())
    }
}
```

在启动构造器中注入：

```rust
let app = FoundryApp::builder()
    .register_hook(AuditHook)
    .build()
    .await?;
```

---
title: "07. Blog Platform Full-Stack Example (blog_platform)"
description: "Stage 7: In-depth analysis of a complete real-world multi-subsystem business application."
---

> **Codebase Navigation**：⬅️ Previous: [06. Facade Layer & Assembly (foundry)](./06-foundry/) · [📋 Overview & Guide](./) · ➡️ Next: [08. Admin Frontend Console (apps/admin)](./08-admin-frontend/)

# 阶段 7：Blog Platform 示例 (examples/blog_platform)

## 概述

`blog_platform` 是一个**完整的示例应用**，演示如何使用 Foundry 框架构建实际业务系统。它展示了：

- **双子系统架构**：Blog (内容发布) + Newsletter (订阅管理)
- **业务逻辑分层**：DTO → Logic → Controllers
- **自定义路由**：在 Auto-CRUD 基础上添加业务接口
- **Hook 机制**：拦截数据变更添加业务逻辑
- **自定义管理页面**：扩展 Admin UI

---

## 1. 项目结构

```
examples/blog_platform/
├── Cargo.toml                   # 依赖配置
├── src/
│   ├── main.rs                  # 应用入口
│   ├── hooks.rs                 # 全局 Hook (BlogMutationHook)
│   └── systems/                 # 业务子系统
│       ├── mod.rs
│       ├── blog/                # 博客子系统
│       │   ├── mod.rs           # BlogSubsystem 定义
│       │   ├── controllers/     # HTTP 路由处理
│       │   ├── dto/             # 数据传输对象
│       │   ├── logic/           # 业务逻辑层
│       │   └── custom_pages/    # 自定义 Admin UI 页面
│       └── newsletter/          # 通讯录子系统
│           ├── mod.rs           # NewsletterSubsystem 定义
│           ├── controllers/
│           ├── dto/
│           ├── logic/
│           └── custom_pages/
```

**架构模式**：每个子系统采用**经典三层架构**
- **DTO** (Data Transfer Object)：请求/响应数据结构 + 验证规则
- **Logic**：纯业务逻辑，不依赖 HTTP
- **Controllers**：HTTP 层，调用 Logic 并处理响应

---

## 2. 应用启动流程 (main.rs)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 从环境变量构建配置
    let config = FoundryConfig::from_env();

    // 2. 使用 Builder 模式初始化应用
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(BlogSubsystem)       // 注册博客子系统
        .register_subsystem(NewsletterSubsystem) // 注册通讯录子系统
        .register_hook(BlogMutationHook)         // 注册全局数据变更钩子
        .build()
        .await?;

    // 3. 启动 HTTP 服务器
    app.run().await?;
    Ok(())
}
```

### 关键步骤

1. **FoundryConfig::from_env()**
   - 从环境变量 `.env` 读取数据库/Redis/JWT 配置
   - 对应 Stage 6 中 `FoundryConfig` 的 10 个字段

2. **register_subsystem()**
   - 注册实现 `SubsystemModule` trait 的业务模块
   - 每个子系统有独立的 `slug`、路由、数据模型

3. **register_hook()**
   - 注册全局 Hook，拦截所有 CRUD 操作
   - 在 Stage 4 中详细分析了 `MutationHook` trait

4. **build() & run()**
   - `build()` 初始化数据库连接、路由、中间件
   - `run()` 启动 Axum HTTP 服务器

---

## 3. 全局 Hook (hooks.rs)

```rust
pub struct BlogMutationHook;

#[async_trait]
impl MutationHook for BlogMutationHook {
    // 创建记录前：拦截 "posts" 模型，修改数据
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if model_slug == "posts" {
            tracing::info!(
                "Intercepted post creation in system '{}' for author: {:?}",
                ctx.system_slug,
                data.get("author")
            );
            // 动态注入字段
            if let Some(obj) = data.as_object_mut() {
                obj.insert("hook_processed".to_string(), serde_json::json!(true));
            }
        }
        Ok(())
    }

    // 创建记录后：记录日志
    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        tracing::info!(
            "Successfully created record {} in system '{}' (model: {})",
            record_id,
            ctx.system_slug,
            model_slug
        );
        Ok(())
    }
}
```

### 实际应用场景

| 场景 | Hook 方法 | 示例 |
|------|-----------|------|
| 字段自动填充 | `before_create` | 自动设置 `created_by`、`ip_address` |
| 数据验证增强 | `before_create/update` | 检查业务规则（如余额不足） |
| 触发外部服务 | `after_create` | 创建订单后发送邮件、推送通知 |
| 审计日志 | `after_update/delete` | 记录敏感操作 |
| 级联操作 | `after_delete` | 删除用户时清理关联数据 |

---

## 4. Blog 子系统 (systems/blog)

### 4.1 SubsystemModule 实现 (blog/mod.rs)

```rust
pub struct BlogSubsystem;

impl SubsystemModule for BlogSubsystem {
    fn slug(&self) -> &'static str {
        "blog"  // 子系统唯一标识，对应 URL /api/v1/s/blog
    }

    fn display_name(&self) -> &'static str {
        "Content & Blog Platform"  // 管理后台显示名称
    }

    fn description(&self) -> &'static str {
        "Articles, publishing workflow, and content management"
    }

    // 注册自定义路由
    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());

        // 提供静态文件服务（自定义管理页面）
        let possible_dirs = [
            PathBuf::from("examples/blog_platform/src/systems/blog/custom_pages"),
            PathBuf::from("src/systems/blog/custom_pages"),
            PathBuf::from("static/custom_pages/blog"),
        ];

        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }

        r
    }

    // 在 Admin UI 中添加自定义页面
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "article_editor".to_string(),
            title: "Article Studio".to_string(),
            icon: "FileEdit".to_string(),
            page_type: "iframe".to_string(),  // 嵌入式页面
            entry: "/api/v1/s/blog/ext/custom-pages/article_editor.html".to_string(),
            required_role: None,  // 无角色限制
        }]
    }
}
```

### 4.2 DTO 层 (blog/dto/mod.rs)

```rust
// 创建文章请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 3, max = 120, message = "Title must be between 3 and 120 chars"))]
    pub title: String,
    
    #[validate(length(min = 5, message = "Content must be at least 5 chars"))]
    pub content: String,
    
    pub author: String,
    
    #[serde(default)]
    pub tags: Vec<String>,
}

// 文章响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}

// 发布文章请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PublishPostRequest {
    pub post_id: u64,
    pub notify_subscribers: bool,  // 是否通知订阅者
}
```

**验证规则**：使用 `validator` crate 的 `#[validate]` 宏
- `length(min, max)`：字符串长度校验
- 自定义 `message`：错误提示信息
- 自动在 Controller 中调用 `payload.validate()?`

### 4.3 Logic 层 (blog/logic/mod.rs)

```rust
pub struct BlogService;

impl BlogService {
    // 创建文章（示例实现）
    pub async fn create_post(
        _ctx: &SystemContext,
        req: CreatePostRequest,
    ) -> AppResult<PostResponse> {
        // 实际应用中会调用 RecordStore 存储到数据库
        // 这里简化为返回模拟数据
        let post = PostResponse {
            id: 1001,
            title: req.title,
            content: req.content,
            author: req.author,
            tags: req.tags,
            published: false,
            created_at: Utc::now(),
        };

        Ok(post)
    }

    // 发布文章
    pub async fn publish_post(
        _ctx: &SystemContext,
        req: PublishPostRequest,
    ) -> AppResult<PostResponse> {
        // 实际应用中：
        // 1. 更新 published 字段
        // 2. 如果 notify_subscribers=true，触发邮件队列
        // 3. 更新缓存、搜索引擎索引等
        
        let post = PostResponse {
            id: req.post_id,
            title: "Published Article: Rust 2026".to_string(),
            content: "Foundry framework architecture walkthrough".to_string(),
            author: "dev@example.com".to_string(),
            tags: vec!["rust".to_string(), "framework".to_string()],
            published: true,
            created_at: Utc::now(),
        };

        Ok(post)
    }
}
```

**设计要点**：
- **无状态**：Service 是无状态的（Rust 的 `struct` 仅用于命名空间）
- **接收 SystemContext**：获取当前子系统 slug、请求 ID、管理员信息
- **返回 AppResult<T>**：统一错误处理（对应 Stage 1 的 `AppError`）

### 4.4 Controllers 层 (blog/controllers/mod.rs)

```rust
pub fn build_routes() -> Router {
    Router::new()
        .route("/posts", post(handle_create_post))
        .route("/posts/publish", post(handle_publish_post))
}

// POST /api/v1/s/blog/ext/posts
pub async fn handle_create_post(
    Extension(ctx): Extension<SystemContext>,  // 从中间件注入
    Json(payload): Json<CreatePostRequest>,
) -> AppResult<Json<ApiResponse<PostResponse>>> {
    payload.validate()?;  // 验证 DTO
    let post = BlogService::create_post(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(post)))  // 统一响应格式
}

// POST /api/v1/s/blog/ext/posts/publish
pub async fn handle_publish_post(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<PublishPostRequest>,
) -> AppResult<Json<ApiResponse<PostResponse>>> {
    payload.validate()?;
    let post = BlogService::publish_post(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(post)))
}
```

**关键技术**：
1. **Extension(SystemContext)**
   - 由 `extract_context` 中间件注入（Stage 5 分析）
   - 包含当前请求的子系统上下文

2. **ApiResponse::success()**
   - Stage 1 中定义的统一响应格式
   - 自动包装为 `{"success":true, "data":{...}}`

3. **AppResult<T>**
   - 自动错误转换（通过 `IntoResponse` trait）
   - 错误会转为 JSON 格式返回

---

## 5. Newsletter 子系统 (systems/newsletter)

### 5.1 结构对比

Newsletter 和 Blog 的结构完全一致，也采用三层架构：

```
newsletter/
├── mod.rs              # NewsletterSubsystem (SubsystemModule 实现)
├── dto/mod.rs          # SubscribeRequest, SubscriberResponse
├── logic/mod.rs        # NewsletterService
└── controllers/mod.rs  # handle_subscribe
```

### 5.2 核心代码

**DTO (newsletter/dto/mod.rs)**：
```rust
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubscribeRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub source: Option<String>,  // 订阅来源（如 "landing_page", "article"）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberResponse {
    pub id: u64,
    pub email: String,
    pub active: bool,
    pub subscribed_at: DateTime<Utc>,
}
```

**Controller (newsletter/controllers/mod.rs)**：
```rust
pub fn build_routes() -> Router {
    Router::new().route("/subscribe", post(handle_subscribe))
}

// POST /api/v1/s/newsletter/ext/subscribe
pub async fn handle_subscribe(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<SubscribeRequest>,
) -> AppResult<Json<ApiResponse<SubscriberResponse>>> {
    payload.validate()?;  // 自动验证邮箱格式
    let result = NewsletterService::subscribe(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

**Logic (newsletter/logic/mod.rs)**：
```rust
impl NewsletterService {
    pub async fn subscribe(
        _ctx: &SystemContext,
        req: SubscribeRequest,
    ) -> AppResult<SubscriberResponse> {
        // 实际应用中：
        // 1. 检查邮箱是否已订阅
        // 2. 存储到 subscribers 表
        // 3. 发送确认邮件（双重确认机制）
        // 4. 触发欢迎邮件
        
        Ok(SubscriberResponse {
            id: 501,
            email: req.email,
            active: true,
            subscribed_at: Utc::now(),
        })
    }
}
```

---

## 6. 路由架构总览

当启动 `blog_platform` 应用后，完整的路由树为：

```
/api/v1/
├── auth/                        # 认证路由（Stage 5）
│   ├── POST /login
│   └── POST /logout
│
├── admin/                       # 管理员 CRUD（Stage 5）
│   ├── GET /admins
│   ├── POST /admins
│   └── ...
│
├── s/blog/                      # Blog 子系统
│   ├── posts/                   # Auto-CRUD（Stage 5）
│   │   ├── GET    /posts        # 列表
│   │   ├── POST   /posts        # 创建
│   │   ├── GET    /posts/:id    # 详情
│   │   ├── PUT    /posts/:id    # 更新
│   │   └── DELETE /posts/:id    # 删除
│   │
│   └── ext/                     # 自定义路由
│       ├── POST /posts          # BlogSubsystem 注册
│       ├── POST /posts/publish
│       └── /custom-pages/*      # 静态文件服务
│
└── s/newsletter/                # Newsletter 子系统
    ├── subscribers/             # Auto-CRUD
    │   ├── GET    /subscribers
    │   └── ...
    │
    └── ext/                     # 自定义路由
        ├── POST /subscribe      # NewsletterSubsystem 注册
        └── /custom-pages/*
```

### 路由注册顺序

1. **FoundryApp 内部路由** (Stage 5 `router.rs`)
   - `/auth/*`：登录、登出
   - `/admin/*`：管理员管理
   - `/systems/*`：系统管理
   - `/models/*`：模型管理

2. **Auto-CRUD 路由** (每个子系统)
   - `/s/:slug/:model`：自动生成的 CRUD API
   - 通过 `HookPipeline` 集成全局 Hook

3. **自定义路由** (SubsystemModule::register_routes)
   - `/s/:slug/ext/*`：业务逻辑路由
   - 如 `/s/blog/ext/posts/publish`

---

## 7. 数据流完整示例

### 场景：创建一篇博客文章

**HTTP 请求**：
```bash
POST /api/v1/s/blog/ext/posts
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "title": "Foundry 架构深度解析",
  "content": "本文详细介绍...",
  "author": "dev@example.com",
  "tags": ["rust", "backend"]
}
```

**执行流程**：

1. **中间件层** (Stage 5)
   ```
   extract_context → audit_middleware → auth_middleware
   ```
   - 提取 JWT，验证权限
   - 将 `SystemContext` 注入到 `Extension`
   - 记录审计日志

2. **路由匹配**
   ```
   /s/blog/ext/posts → BlogSubsystem 的 controllers::handle_create_post
   ```

3. **Controller 层**
   ```rust
   Json(payload): Json<CreatePostRequest>  // 反序列化
   payload.validate()?                     // 验证 title 长度、content 长度
   ```

4. **Logic 层**
   ```rust
   BlogService::create_post(&ctx, payload).await?
   ```
   - 实际应用中调用 `RecordStore` 存储到 PostgreSQL
   - 触发 `BlogMutationHook::before_create`（注入 `hook_processed` 字段）

5. **Hook 拦截**
   ```rust
   BlogMutationHook::before_create
   → 修改 data 添加额外字段
   
   BlogMutationHook::after_create
   → 记录日志 "Successfully created record 1001"
   ```

6. **响应返回**
   ```json
   {
     "success": true,
     "data": {
       "id": 1001,
       "title": "Foundry 架构深度解析",
       "content": "本文详细介绍...",
       "author": "dev@example.com",
       "tags": ["rust", "backend"],
       "published": false,
       "created_at": "2026-09-15T10:30:00Z"
     }
   }
   ```

---

## 8. 实际应用扩展点

### 8.1 替换示例逻辑为真实数据库操作

在 `BlogService::create_post` 中：

```rust
pub async fn create_post(
    ctx: &SystemContext,
    req: CreatePostRequest,
) -> AppResult<PostResponse> {
    // 获取 RecordStore 实例（从 AppState）
    let record_store = ctx.get_record_store();
    
    // 构造 JSONB 数据
    let data = serde_json::json!({
        "title": req.title,
        "content": req.content,
        "author": req.author,
        "tags": req.tags,
        "published": false,
        "created_at": Utc::now(),
    });
    
    // 调用 RecordStore::create（会触发 Hook）
    let record_id = record_store
        .create(ctx, "posts", data)
        .await?;
    
    // 查询完整记录返回
    let record = record_store
        .find_by_id(ctx, "posts", record_id)
        .await?;
    
    Ok(serde_json::from_value(record)?)
}
```

### 8.2 添加更多业务接口

在 `blog/controllers/mod.rs` 中：

```rust
pub fn build_routes() -> Router {
    Router::new()
        .route("/posts", post(handle_create_post))
        .route("/posts/publish", post(handle_publish_post))
        .route("/posts/:id/comments", get(list_comments))    // 新增
        .route("/posts/:id/like", post(like_post))            // 新增
        .route("/drafts", get(list_drafts))                   // 新增
}
```

### 8.3 集成第三方服务

在 `NewsletterService::subscribe` 中：

```rust
pub async fn subscribe(
    ctx: &SystemContext,
    req: SubscribeRequest,
) -> AppResult<SubscriberResponse> {
    // 1. 存储到数据库
    let subscriber = store_subscriber(ctx, &req.email).await?;
    
    // 2. 发送到 SendGrid API
    send_welcome_email(&req.email).await?;
    
    // 3. 同步到 Mailchimp
    sync_to_mailchimp(&req.email).await?;
    
    Ok(subscriber)
}
```

---

## 9. 关键设计模式总结

### 9.1 三层架构优势

| 层次 | 职责 | 可测试性 | 复用性 |
|------|------|---------|--------|
| **DTO** | 数据验证、序列化 | 单元测试验证规则 | 可复用于 gRPC、GraphQL |
| **Logic** | 纯业务逻辑 | 无需 HTTP 环境测试 | 可被多个 Controller 调用 |
| **Controllers** | HTTP 路由、响应格式 | 集成测试 API | 专注于协议层 |

### 9.2 依赖注入

```rust
Extension(ctx): Extension<SystemContext>
```
- 不在函数内部创建依赖（如数据库连接）
- 通过中间件注入，便于测试时 Mock

### 9.3 错误传播

```rust
payload.validate()?;           // ValidationError → AppError
let post = Service::create(...).await?;  // 自动传播
Ok(Json(ApiResponse::success(post)))     // 成功时包装
```
- Rust 的 `?` 操作符自动转换错误类型
- `AppError` 实现了 `From<ValidationError>` 等 trait

---

## 10. 与前 6 个阶段的关联

| 阶段 | Blog Platform 如何使用 |
|------|------------------------|
| **Stage 1 (Core)** | `SystemContext` 在 Controller 中注入，`AppError`/`ApiResponse` 统一处理响应 |
| **Stage 2 (Storage)** | 实际应用中 `BlogService` 会调用 `RecordStore` 存储到 PostgreSQL |
| **Stage 3 (Auth)** | 所有路由经过 `auth_middleware`，验证 JWT 和 RBAC 角色 |
| **Stage 4 (Extension)** | `BlogMutationHook` 拦截 Auto-CRUD 操作，注入业务逻辑 |
| **Stage 5 (Engine)** | 继承所有中间件（audit/auth/context）、Auto-CRUD 路由 |
| **Stage 6 (Foundry)** | 通过 `FoundryApp::builder()` 注册子系统和 Hook |

---

## 11. 学习检查点

掌握 Blog Platform 后，你应该能够：

✅ **理解**：如何使用 `SubsystemModule` trait 定义业务模块  
✅ **理解**：三层架构（DTO/Logic/Controllers）的职责划分  
✅ **掌握**：如何在 `register_routes` 中添加自定义路由  
✅ **掌握**：如何通过 `MutationHook` 拦截数据变更  
✅ **掌握**：如何使用 `Extension<SystemContext>` 获取上下文  
✅ **实践**：为 Blog 子系统添加新的 API 端点（如评论功能）  
✅ **实践**：创建第三个子系统（如 `UserProfile`）  
✅ **实践**：集成第三方服务（如 SendGrid 邮件、OSS 对象存储）

---

## 12. 下一步：Stage 8 - Admin 前端

接下来分析 `apps/admin`（React + TailwindCSS 前端），了解：
- 如何调用 Foundry API
- 自动生成的 CRUD 界面
- 自定义管理页面的集成
- 完整的用户体验流程

---

**Stage 7 完成！** 你已经完整理解了 Foundry 框架的实际应用方式。

---

### 📚 Stage Navigation
- ⬅️ **Previous Stage**: [06. Facade Layer & Assembly (foundry)](./06-foundry/)
- 📋 **Guide Overview**: [Return to Overview](./)
- ➡️ **Next Stage**: [08. Admin Frontend Console (apps/admin)](./08-admin-frontend/)

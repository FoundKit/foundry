---
title: "05. 请求引擎与自动 CRUD (foundry_engine)"
description: "阶段 5：深度剖析 Axum 路由装配、中间件栈、Auto-CRUD 引擎与审计日志。"
---

> **源码导读导航**：⬅️ 上一篇：[04. 扩展系统与生命周期钩子 (foundry_extension)](./04-extension/) · [📋 学习指南与总览](./) · ➡️ 下一篇：[06. 门面层与应用组装 (foundry)](./06-foundry/)

# Foundry Engine 代码深度分析

> 第五阶段：理解请求引擎（路由、处理器、中间件、Auto-CRUD）
>
> Crate: `foundry_engine`
>
> 阅读时间：60-90 分钟

---

## 概述

`foundry_engine` 是 Foundry 框架的**HTTP 请求处理引擎**，实现了：
- **统一路由系统**（管理 API + Auto-CRUD + 自定义路由）
- **Auto-CRUD 处理器**（零代码 RESTful CRUD）
- **中间件系统**（认证、上下文提取、审计）
- **应用状态管理**
- **静态资源服务**（Admin UI）

**这是框架的"大脑"，将所有底层组件整合在一起。**

**依赖关系：**
```
foundry_core, foundry_storage, foundry_auth, foundry_extension
                    ↓
              foundry_engine (组装)
```

**关键依赖：**
- `axum` - Web 框架（路由、处理器）
- `tower` - 中间件基础设施
- `tower-http` - CORS、Trace、ServeDir
- `rust-embed` - 静态资源嵌入

---

## 架构总览

```
foundry_engine
├── state.rs          # AppState：应用全局状态
├── router.rs         # 路由树构建
├── middleware/       # 中间件
│   ├── auth.rs       # 认证中间件
│   ├── context.rs    # 上下文提取
│   ├── audit.rs      # 审计日志
│   └── mod.rs
├── handlers/         # HTTP 处理器
│   ├── autocrud.rs   # ⭐ Auto-CRUD 核心
│   ├── auth.rs       # 登录/注册
│   ├── systems.rs    # 子系统管理
│   ├── models.rs     # 模型管理
│   ├── configs.rs    # 配置管理
│   ├── admins.rs     # 管理员管理
│   ├── audit.rs      # 审计日志查询
│   └── mod.rs
├── external.rs       # 外部子系统加载
└── lib.rs            # 统一导出
```

---

## 1. state.rs - 应用状态

### 1.1 AppState 结构

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,                                  // PostgreSQL 连接池
    pub redis: Option<RedisPool>,                    // Redis 连接池（可选）
    pub jwt: Arc<JwtService>,                        // JWT 服务
    pub hooks: Arc<HookPipeline>,                    // 钩子管道
    pub subsystems: Arc<Vec<Box<dyn SubsystemModule>>>, // 注册的子系统
}
```

**设计要点：**

#### A. Clone 派生

```rust
#[derive(Clone)]
pub struct AppState
```

**原因：** Axum 的 `State<AppState>` 需要在每个请求中克隆状态。

**性能：** 所有字段都是 `Arc` 或轻量级类型，克隆开销极低（只增加引用计数）。

#### B. Arc 包装

```rust
pub jwt: Arc<JwtService>,
pub hooks: Arc<HookPipeline>,
pub subsystems: Arc<Vec<Box<dyn SubsystemModule>>>,
```

**原因：**
- 多线程共享（`Arc`）
- 不可变引用（读多写少）
- 避免重复分配内存

#### C. 可选 Redis

```rust
pub redis: Option<RedisPool>
```

**灵活性：** Redis 不是必须的，可以在开发环境省略。

### 1.2 构造方法

```rust
impl AppState {
    pub fn new(
        db: DbPool,
        redis: Option<RedisPool>,
        jwt: JwtService,
        hooks: HookPipeline,
        subsystems: Vec<Box<dyn SubsystemModule>>,
    ) -> Self {
        Self {
            db,
            redis,
            jwt: Arc::new(jwt),
            hooks: Arc::new(hooks),
            subsystems: Arc::new(subsystems),
        }
    }
}
```

**使用示例：**

```rust
let state = AppState::new(
    db_pool,
    Some(redis_pool),
    JwtService::new("secret", 24),
    hook_pipeline,
    vec![
        Box::new(BlogSubsystem),
        Box::new(ShopSubsystem),
    ],
);
```

---

## 2. router.rs - 路由系统

### 2.1 路由树结构

```
/
├── /                            → Redirect to /admin/
├── /__health                    → Health check
├── /admin/*                     → Admin UI (SPA)
├── /assets/*                    → Embedded assets
└── /api/v1/
    ├── /admin/                  → Admin Control Plane
    │   ├── /auth/login          → 登录（公开）
    │   ├── /auth/me             → 当前用户（需认证）
    │   ├── /platform/summary    → 平台统计
    │   ├── /systems             → 子系统 CRUD
    │   ├── /admins              → 管理员 CRUD
    │   ├── /audit-logs          → 审计日志
    │   └── /s/:slug/*           → 子系统管理（需 Topic 权限）
    │       ├── /details         → 子系统详情
    │       ├── /stats           → 子系统统计
    │       ├── /configs/*       → 配置管理
    │       └── /models/*        → 模型管理
    ├── /s/:slug/*               → Auto-CRUD + Custom APIs
    │   ├── /configs             → 配置读写
    │   ├── /:model              → Auto-CRUD 列表/创建
    │   ├── /:model/:id          → Auto-CRUD 查询/更新/删除
    │   └── /ext/*               → 自定义子系统 API
    └── /health                  → Health check
```

### 2.2 路由构建代码

```rust
pub fn build_router(state: AppState) -> Router {
    // 1. Admin API（管理后台）
    let admin_public_routes = Router::new()
        .route("/auth/login", post(auth::login_handler));

    let admin_protected_routes = Router::new()
        .route("/auth/me", get(auth::me_handler))
        .route("/platform/summary", get(systems::get_platform_summary_handler))
        // ... 其他管理路由
        .route_layer(from_fn_with_state(state.clone(), require_admin_auth));

    let admin_api = Router::new()
        .merge(admin_public_routes)
        .merge(admin_protected_routes);

    // 2. Auto-CRUD API（动态 REST CRUD）
    let autocrud_api = Router::new()
        .route(
            "/s/{system_slug}/{model_slug}",
            get(autocrud::list_records_handler)
                .post(autocrud::create_record_handler),
        )
        .route(
            "/s/{system_slug}/{model_slug}/{id}",
            get(autocrud::get_record_handler)
                .put(autocrud::update_record_handler)
                .delete(autocrud::delete_record_handler),
        );

    // 3. 自定义子系统 API
    let mut custom_ext_api = Router::new();
    for sub in state.subsystems.iter() {
        let sub_router = sub.register_routes(Router::new());
        custom_ext_api = custom_ext_api.nest_service(
            &format!("/s/{}/ext", sub.slug()),
            sub_router
        );
    }

    // 4. 组装完整路由树
    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .nest("/admin", admin_api)
        .merge(autocrud_api)
        .merge(custom_ext_api);

    // 5. 应用全局中间件
    let router = Router::new()
        .nest("/api/v1", api_v1)
        .layer(from_fn_with_state(state.clone(), audit_interceptor))    // 审计
        .layer(from_fn_with_state(state.clone(), extract_system_context)) // 上下文
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))   // CORS
        .layer(TraceLayer::new_for_http())                              // 日志
        .with_state(state);

    // 6. Admin UI 静态资源
    router = mount_admin_ui(router);

    router
}
```

### 2.3 中间件层级

```
HTTP 请求
    ↓
[TraceLayer]           ← 日志追踪
    ↓
[CorsLayer]            ← 跨域处理
    ↓
[extract_system_context] ← 提取上下文（system_slug, locale, IP）
    ↓
[audit_interceptor]    ← 审计日志拦截
    ↓
[require_admin_auth]   ← 管理员认证（特定路由）
    ↓
[require_topic_access] ← Topic 权限检查（特定路由）
    ↓
Handler                ← 业务逻辑处理
    ↓
Response
```

**执行顺序：** 从外到内（layer 的顺序与执行顺序相反）

---

## 3. middleware - 中间件系统

### 3.1 context.rs - 上下文提取

#### 功能：提取租户、语言、IP 等信息

```rust
pub async fn extract_system_context(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();

    // 1. 从路径提取 system_slug
    let mut detected_slug = None;
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    // 匹配 /api/v1/s/{system_slug}/*
    if segments.len() >= 4 && segments[0] == "api" 
       && segments[1] == "v1" && segments[2] == "s" {
        detected_slug = Some(segments[3].to_string());
    }
    
    // 匹配 /api/v1/admin/s/{system_slug}/*
    else if segments.len() >= 5 && segments[0] == "api" 
            && segments[1] == "v1" && segments[2] == "admin" 
            && segments[3] == "s" {
        detected_slug = Some(segments[4].to_string());
    }

    // 2. Fallback：从 Header 提取
    if detected_slug.is_none() {
        if let Some(header_val) = req.headers().get("x-foundry-system-id") {
            if let Ok(slug_str) = header_val.to_str() {
                detected_slug = Some(slug_str.to_string());
            }
        }
    }

    // 3. 提取语言
    let locale = req
        .headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .unwrap_or("en-US")
        .to_string();

    // 4. 提取客户端 IP
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    // 5. 提取 User-Agent
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 6. 构建 SystemContext
    let slug = detected_slug.unwrap_or_else(|| "default".to_string());
    let ctx = SystemContext::with_details(
        None, &slug, &slug, &locale, client_ip, user_agent
    );

    // 7. 注入到请求扩展
    req.extensions_mut().insert(ctx);
    req.extensions_mut().insert(_state.db.clone());
    if let Some(ref redis) = _state.redis {
        req.extensions_mut().insert(redis.clone());
    }

    next.run(req).await
}
```

**提取优先级：**
1. URL 路径（`/api/v1/s/{slug}`）
2. HTTP Header（`X-Foundry-System-ID`）
3. 默认值（`"default"`）

**注入内容：**
- `SystemContext`
- `DbPool`
- `RedisPool`（可选）

### 3.2 auth.rs - 认证中间件

#### A. require_admin_auth - 管理员认证

```rust
pub async fn require_admin_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 Authorization Header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    // 2. 提取 Bearer Token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization scheme".into()))?;

    // 3. 验证 JWT
    let claims = state.jwt.verify_token(token)?;

    // 4. 注入 Claims 到请求扩展
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
```

**应用场景：**
```rust
let admin_protected_routes = Router::new()
    .route("/systems", get(list_systems))
    .route_layer(from_fn_with_state(state.clone(), require_admin_auth));
```

#### B. require_topic_access - Topic 权限检查

```rust
pub async fn require_topic_access(
    req: Request,
    next: Next
) -> Result<Response, AppError> {
    // 1. 获取 AdminClaims（由 require_admin_auth 注入）
    let claims = req
        .extensions()
        .get::<AdminClaims>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".into()))?;

    // 2. 获取 SystemContext（由 extract_system_context 注入）
    let ctx = req
        .extensions()
        .get::<SystemContext>()
        .ok_or_else(|| AppError::Internal("SystemContext missing".into()))?;

    // 3. 检查权限
    check_system_access(claims, &ctx.system_slug)?;

    Ok(next.run(req).await)
}
```

**权限逻辑：**
- Super Admin / General Admin → 全部通过
- Topic Admin → 检查 `allowed_systems` 白名单

### 3.3 audit.rs - 审计日志拦截器

```rust
pub async fn audit_interceptor(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    // 提取管理员信息
    let claims = req.extensions().get::<AdminClaims>().cloned();
    let ctx = req.extensions().get::<SystemContext>().cloned();

    // 执行请求
    let response = next.run(req).await;
    
    // 记录审计日志
    let duration_ms = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i16;

    if let (Some(claims), Some(ctx)) = (claims, ctx) {
        let log = AuditLogInsert {
            admin_id: Some(claims.sub),
            admin_username: Some(claims.username),
            system_slug: Some(ctx.system_slug),
            method,
            path,
            status_code: Some(status_code),
            duration_ms: Some(duration_ms),
            // ...
        };
        
        // 异步记录（不阻塞响应）
        tokio::spawn(async move {
            if let Err(e) = AuditStore::insert(&state.db, log).await {
                tracing::error!("Failed to log audit: {}", e);
            }
        });
    }

    response
}
```

---

## 4. handlers/autocrud.rs - Auto-CRUD 核心 ⭐

### 4.1 设计理念

**零代码 CRUD：** 只需定义模型（Model + Fields），自动生成 RESTful API。

```
定义模型
    ↓
POST /api/v1/admin/s/blog/models
{
  "slug": "articles",
  "name": "文章"
}
    ↓
POST /api/v1/admin/s/blog/models/1/fields
{
  "name": "title",
  "field_type": "string",
  "is_required": true
}
    ↓
自动生成 5 个 API：
GET    /api/v1/s/blog/articles           (列表)
POST   /api/v1/s/blog/articles           (创建)
GET    /api/v1/s/blog/articles/:id       (查询)
PUT    /api/v1/s/blog/articles/:id       (更新)
DELETE /api/v1/s/blog/articles/:id       (删除)
```

### 4.2 创建记录（POST）

```rust
pub async fn create_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    // 1. 验证模型存在
    let model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    
    // 2. 获取字段定义
    let fields = ModelStore::list_fields(&state.db, model.id).await?;

    // 3. 内存中验证数据（类型、必填）
    RecordStore::validate_record(&fields, &payload)?;

    // 4. 执行 before_create 钩子
    state.hooks.execute_before_create(&ctx, &model_slug, &mut payload).await?;

    // 5. 数据库插入
    let record = RecordStore::create(
        &state.db,
        &system_slug,
        &model_slug,
        payload.clone()
    ).await?;

    // 6. 执行 after_create 钩子
    state.hooks.execute_after_create(&ctx, &model_slug, record.id, &payload).await?;

    Ok(Json(ApiResponse::success(record)))
}
```

**完整流程图：**

```
HTTP POST /api/v1/s/blog/articles
    ↓
[extract_system_context]       ← 提取 SystemContext
    ↓
[require_admin_auth]           ← 验证 JWT（可选）
    ↓
create_record_handler
    ↓
1. 验证模型存在
    ↓
2. 获取字段定义
    ↓
3. validate_record              ← 验证必填、类型
    ↓
4. execute_before_create        ← 钩子拦截（可修改 payload）
    ↓
5. RecordStore::create          ← 数据库 INSERT
    ↓
6. execute_after_create         ← 钩子拦截（发送通知、更新缓存等）
    ↓
ApiResponse::success(record)
    ↓
[audit_interceptor]            ← 记录审计日志
    ↓
HTTP 201 Created
```

### 4.3 更新记录（PUT）

```rust
pub async fn update_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    let fields = ModelStore::list_fields(&state.db, model.id).await?;

    RecordStore::validate_record(&fields, &payload)?;

    state.hooks.execute_before_update(&ctx, &model_slug, id, &mut payload).await?;

    let record = RecordStore::update(
        &state.db,
        &system_slug,
        &model_slug,
        id,
        payload.clone()
    ).await?;

    state.hooks.execute_after_update(&ctx, &model_slug, id, &payload).await?;

    Ok(Json(ApiResponse::success(record)))
}
```

### 4.4 删除记录（DELETE）

```rust
pub async fn delete_record_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;

    state.hooks.execute_before_delete(&ctx, &model_slug, id).await?;

    RecordStore::delete(&state.db, &system_slug, &model_slug, id).await?;

    state.hooks.execute_after_delete(&ctx, &model_slug, id).await?;

    Ok(Json(ApiResponse::success(())))
}
```

### 4.5 查询记录（GET）

#### A. 列表查询

```rust
pub async fn list_records_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Query(query): Query<RecordQuery>,  // ← page, page_size, sort_by
) -> AppResult<Json<ApiResponse<PaginatedData<ModelRecordEntity>>>> {
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;

    let records = RecordStore::list(&state.db, &system_slug, &model_slug, query).await?;

    Ok(Json(ApiResponse::success(records)))
}
```

**请求示例：**
```bash
GET /api/v1/s/blog/articles?page=1&page_size=20&sort_by=created_at
```

**响应：**
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "items": [
      {
        "id": 1,
        "system_id": "blog",
        "model_slug": "articles",
        "data": { "title": "Hello", "content": "..." },
        "created_at": "2026-09-15T10:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "page_size": 20,
      "total": 100,
      "total_pages": 5
    }
  }
}
```

#### B. 单条查询

```rust
pub async fn get_record_handler(
    State(state): State<AppState>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    let _model = ModelStore::get_model(&state.db, &system_slug, &model_slug).await?;
    
    let record = RecordStore::get_by_id(&state.db, &system_slug, &model_slug, id).await?;
    
    Ok(Json(ApiResponse::success(record)))
}
```

---

## 5. 静态资源服务

### 5.1 Admin UI 嵌入

```rust
#[derive(rust_embed::Embed)]
#[folder = "admin_dist"]
struct EmbeddedAdminAssets;

async fn serve_embedded_file(path: &str) -> Response {
    let clean_path = path.trim_start_matches('/');
    let target = if clean_path.is_empty() { "index.html" } else { clean_path };

    if let Some(file) = EmbeddedAdminAssets::get(target) {
        let mime = mime_guess::from_path(target).first_or_octet_stream();
        return ([(CONTENT_TYPE, mime.as_ref())], file.data).into_response();
    }

    // SPA fallback：所有未找到的路径返回 index.html
    if let Some(index) = EmbeddedAdminAssets::get("index.html") {
        return ([(CONTENT_TYPE, "text/html")], index.data).into_response();
    }

    (StatusCode::NOT_FOUND, "Admin UI not found").into_response()
}
```

**工作原理：**
1. 编译时将 `admin_dist/` 目录嵌入二进制文件
2. 运行时直接从内存提供文件
3. 未找到文件 → 返回 `index.html`（支持 SPA 客户端路由）

### 5.2 磁盘 vs 嵌入

```rust
// 优先从磁盘加载（开发环境）
let admin_static_dirs = [
    "static/admin",
    "apps/admin/dist",
    "../apps/admin/dist",
];

for dir in admin_static_dirs {
    if dir.exists() {
        router = router.nest_service("/admin", ServeDir::new(dir));
        break;
    }
}

// 回退到嵌入资源（生产环境）
if !mounted_disk_admin {
    router = router.route("/admin/{*path}", get(admin_embedded_handler));
}
```

---

## 6. 关键设计模式总结

### 6.1 中间件管道（Middleware Pipeline）

```rust
Router::new()
    .layer(TraceLayer)              // 最外层
    .layer(CorsLayer)
    .layer(from_fn(extract_context))
    .layer(from_fn(audit))
    .route("/api", handler)         // 最内层
```

**执行顺序：** 洋葱模型（从外到内，再从内到外）

### 6.2 依赖注入（Dependency Injection）

```rust
async fn handler(
    State(state): State<AppState>,        // 注入全局状态
    Extension(ctx): Extension<SystemContext>, // 注入上下文
    Extension(claims): Extension<AdminClaims>, // 注入认证信息
) -> Result<Response> {
    // 使用注入的依赖
}
```

### 6.3 约定优于配置（Convention over Configuration）

```
模型定义：/api/v1/admin/s/:slug/models
         ↓ 自动生成
CRUD API：/api/v1/s/:slug/:model
```

无需手动编写路由和处理器。

---

## 7. 性能优化

### 7.1 异步非阻塞

```rust
// ✅ 所有 I/O 操作都是异步的
async fn handler(...) -> AppResult<...> {
    let records = RecordStore::list(&db, ...).await?;  // 非阻塞
    Ok(...)
}
```

### 7.2 连接池复用

```rust
// ✅ DbPool 和 RedisPool 在所有请求间共享
pub struct AppState {
    pub db: DbPool,    // ← 连接池
    pub redis: Option<RedisPool>,
}
```

### 7.3 审计日志异步

```rust
// ✅ 不阻塞响应
tokio::spawn(async move {
    AuditStore::insert(&db, log).await;
});
```

---

## 8. 安全措施

### 8.1 JWT 验证

```rust
let claims = state.jwt.verify_token(token)?;
```

### 8.2 RBAC 检查

```rust
check_system_access(&claims, &system_slug)?;
```

### 8.3 CORS 配置

```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

**生产环境建议：** 限制 `allow_origin` 为特定域名。

---

## 9. 与其他 Crates 的交互

```
foundry_engine
    ↓ 使用
┌────────────────────────────────────┐
│ foundry_core                       │
│   - AppError, ApiResponse          │
│   - SystemContext, SubsystemModule │
├────────────────────────────────────┤
│ foundry_storage                    │
│   - RecordStore, ModelStore        │
│   - SystemStore, AdminStore        │
├────────────────────────────────────┤
│ foundry_auth                       │
│   - JwtService, check_system_access│
├────────────────────────────────────┤
│ foundry_extension                  │
│   - HookPipeline                   │
└────────────────────────────────────┘
```

---

## 10. 快速参考

### 创建自定义路由

```rust
impl SubsystemModule for MySubsystem {
    fn register_routes(&self, router: Router) -> Router {
        router
            .route("/stats", get(my_stats_handler))
            .route("/export", post(my_export_handler))
    }
}
```

### 访问注入的依赖

```rust
async fn my_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<SystemContext>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<MyData>>> {
    // ...
}
```

---

**✅ 第五阶段完成！** 你现在应该理解了：
- AppState 的设计和作用
- 路由树的完整结构（Admin + Auto-CRUD + Custom）
- 中间件系统（认证、上下文、审计）
- Auto-CRUD 的完整实现（验证 + 钩子 + CRUD）
- 静态资源服务（嵌入 vs 磁盘）
- 关键设计模式（中间件管道、依赖注入、约定优于配置）

**准备好进入第六阶段了吗？** 下一步我们将分析 `foundry` 门面层，看所有组件如何组装启动。
---

### 📚 阶段导航
- ⬅️ **上一阶段**：[04. 扩展系统与生命周期钩子 (foundry_extension)](./04-extension/)
- 📋 **学习指南总览**：[返回源码深度理解总览](./)
- ➡️ **下一阶段**：[06. 门面层与应用组装 (foundry)](./06-foundry/)

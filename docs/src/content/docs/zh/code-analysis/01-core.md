---
title: "01. 核心类型与接口定义 (foundry_core)"
description: "阶段 1：深度剖析 Context、Error、SubsystemModule trait 等核心基础设施抽象。"
---

> **源码导读导航**：⬅️ 上一篇：[全景学习指南与代码导读](./) · [📋 学习指南与总览](./) · ➡️ 下一篇：[02. 存储层实现与动态模型 (foundry_storage)](./02-storage/)

# Foundry Core 代码深度分析

> 第一阶段：理解核心类型与接口定义
>
> Crate: `foundry_core`
>
> 阅读时间：30-60 分钟

---

## 概述

`foundry_core` 是整个 Foundry 框架的**基石**，定义了：
- **上下文传递**（SystemContext）
- **错误处理**（AppError、ErrorEnvelope）
- **响应格式**（ApiResponse、PaginatedData）
- **核心接口**（SubsystemModule trait）
- **通用类型**（FieldType、SystemStatus、slug 验证）

**依赖关系：** 被所有其他 crates 依赖，自身只依赖基础库（serde、uuid、axum、thiserror、validator）

---

## 1. context.rs - 上下文传递机制

### 1.1 SystemContext 结构体

```rust
pub struct SystemContext {
    pub system_id: Option<Uuid>,      // 子系统的不可变 UUID（数据库主键）
    pub system_slug: String,           // URL/代码中的唯一标识（如 "blog"）
    pub system_name: String,           // 显示名称（如 "博客系统"）
    pub locale: String,                // 区域设置（如 "zh-CN"）
    pub client_ip: Option<String>,     // 客户端 IP
    pub user_agent: Option<String>,    // User-Agent
}
```

### 1.2 设计要点

#### A. 为什么同时有 `system_id` 和 `system_slug`？

```
system_id (UUID):
  - 数据库主键（不可变）
  - 用于数据库 JOIN 和外键引用
  - 可选（某些场景下还未分配 ID）

system_slug (String):
  - URL 友好的字符串标识
  - 用于路由匹配（/api/v1/s/blog/...）
  - 用户可读（日志、调试）
  - 必需（唯一标识符）
```

#### B. 租户隔离（Multi-tenancy）

**核心理念：** 每个请求都绑定一个 `SystemContext`，确保数据按子系统严格隔离。

```rust
// 示例：创建记录时自动隔离
RecordStore::create(
    &db,
    &ctx.system_slug,  // ← 强制指定子系统
    "articles",
    data
).await?;

// 数据库查询自动加过滤条件：
// SELECT * FROM records 
// WHERE system_slug = 'blog' AND model_slug = 'articles'
```

#### C. Redis Key 辅助方法

```rust
pub fn redis_key(&self, key: &str) -> String {
    format!("foundry:{}:{}", self.system_slug, key)
}

// 使用示例：
let session_key = ctx.redis_key("session:abc123");
// 结果：foundry:blog:session:abc123
```

**优势：**
- 自动加前缀，避免键冲突
- 统一命名规范
- 易于批量操作（KEYS foundry:blog:*）

### 1.3 工厂方法

#### 简单创建（开发/测试）
```rust
let ctx = SystemContext::new("blog");
// 只设置 slug，其他字段使用默认值
```

#### 完整创建（生产环境）
```rust
let ctx = SystemContext::with_details(
    Some(system_uuid),
    "blog",
    "博客系统",
    "zh-CN",
    Some("192.168.1.100".to_string()),
    Some("Mozilla/5.0 ...".to_string())
);
```

### 1.4 在请求中的传递路径

```
HTTP 请求
   ↓
中间件提取路径参数 (:system_slug)
   ↓
查询数据库获取 system_id 和 system_name
   ↓
提取 Accept-Language、X-Real-IP、User-Agent
   ↓
构建 SystemContext
   ↓
通过 Axum Extension 注入到 Handler
   ↓
Handler 函数接收 Extension<SystemContext>
```

---

## 2. error.rs - 统一错误处理

### 2.1 ErrorEnvelope - 国际化错误响应

```rust
pub struct ErrorEnvelope {
    pub code: u32,                    // 错误码（40000、40100...）
    pub message: String,               // 英文错误消息
    pub i18n_key: String,              // 国际化键（errors.bad_request）
    pub args: Option<HashMap<...>>,    // 可选：动态参数
}
```

**设计亮点：**
- **结构化错误码**：便于客户端分类处理
- **i18n_key**：前端可根据用户语言显示本地化消息
- **args**：支持动态参数插值（如 "用户 {username} 不存在"）

### 2.2 AppError - 核心错误枚举

```rust
pub enum AppError {
    BadRequest(String),      // 400 - 错误的请求参数
    Validation(String),       // 422 - 验证失败
    Unauthorized(String),     // 401 - 未认证
    Forbidden(String),        // 403 - 无权限
    NotFound(String),         // 404 - 资源不存在
    Conflict(String),         // 409 - 冲突（如唯一键冲突）
    Internal(String),         // 500 - 服务器内部错误
    Database(String),         // 500 - 数据库错误
}
```

### 2.3 错误码映射表

| AppError 变体 | HTTP 状态码 | 错误码 | i18n_key |
|--------------|-----------|-------|----------|
| BadRequest | 400 | 40000 | errors.bad_request |
| Validation | 422 | 42200 | errors.validation_failed |
| Unauthorized | 401 | 40100 | errors.unauthorized |
| Forbidden | 403 | 40300 | errors.forbidden |
| NotFound | 404 | 40400 | errors.not_found |
| Conflict | 409 | 40900 | errors.conflict |
| Internal | 500 | 50000 | errors.internal_server_error |
| Database | 500 | 50001 | errors.database_error |

### 2.4 实现 IntoResponse（Axum 集成）

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.envelope());
        (status, body).into_response()
    }
}
```

**效果：** Handler 可以直接返回 `Result<T, AppError>`，Axum 自动将错误转换为 HTTP 响应。

```rust
// Handler 示例
async fn get_article(id: i64) -> AppResult<Json<Article>> {
    let article = ArticleStore::get(id)
        .await
        .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    Ok(Json(article))
}

// 错误时自动返回：
// HTTP/1.1 404 Not Found
// Content-Type: application/json
// {
//   "code": 40400,
//   "message": "Not Found: Article not found",
//   "i18n_key": "errors.not_found",
//   "args": null
// }
```

### 2.5 自动转换（From trait）

```rust
// validator crate 的验证错误自动转换
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

// serde_json 解析错误自动转换
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON parsing error: {}", err))
    }
}

// 使用示例：
let data: MyDto = serde_json::from_str(input)?; // ← 自动转换为 AppError
validator::validate(&data)?;                    // ← 自动转换为 AppError
```

### 2.6 AppResult 类型别名

```rust
pub type AppResult<T> = Result<T, AppError>;

// 使用示例：
async fn create_article(dto: CreateArticleDto) -> AppResult<Article> {
    // 函数签名清晰，无需重复写 Result<T, AppError>
}
```

---

## 3. response.rs - 统一响应格式

### 3.1 ApiResponse - 成功响应包装

```rust
pub struct ApiResponse<T> {
    pub code: u32,                    // 成功时为 0
    pub message: String,               // "success" 或自定义消息
    pub data: T,                       // 实际数据（泛型）
    pub meta: Option<Value>,           // 可选元数据
}
```

**设计理念：** 成功和失败响应使用统一结构，客户端根据 `code` 判断。

```json
// 成功响应示例
{
  "code": 0,
  "message": "success",
  "data": {
    "id": 1,
    "title": "Hello World"
  },
  "meta": null
}

// 错误响应示例（ErrorEnvelope）
{
  "code": 40400,
  "message": "Not Found: Article not found",
  "i18n_key": "errors.not_found",
  "args": null
}
```

### 3.2 构造方法

```rust
// 1. 基础成功响应
ApiResponse::success(article)

// 2. 自定义消息
ApiResponse::with_message(article, "Article created successfully")

// 3. 附加元数据
ApiResponse::with_meta(
    articles,
    json!({ "took_ms": 23, "cache_hit": true })
)
```

### 3.3 PageMeta - 分页元数据

```rust
pub struct PageMeta {
    pub page: u64,         // 当前页码（从 1 开始）
    pub page_size: u64,    // 每页条数
    pub total: u64,        // 总记录数
    pub total_pages: u64,  // 总页数（自动计算）
}
```

**自动计算总页数：**
```rust
impl PageMeta {
    pub fn new(page: u64, page_size: u64, total: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            total.div_ceil(page_size)  // ← 向上取整
        };
        // ...
    }
}

// 示例：
PageMeta::new(1, 20, 100) → total_pages = 5
PageMeta::new(1, 20, 101) → total_pages = 6
```

### 3.4 PaginatedData - 分页数据包装

```rust
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub pagination: PageMeta,
}

// 使用示例：
let articles = ArticleStore::list(page, page_size).await?;
let total = ArticleStore::count().await?;

let paginated = PaginatedData::new(articles, page, page_size, total);
Ok(Json(ApiResponse::success(paginated)))

// 响应示例：
{
  "code": 0,
  "message": "success",
  "data": {
    "items": [
      { "id": 1, "title": "Article 1" },
      { "id": 2, "title": "Article 2" }
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

---

## 4. types.rs - 通用类型定义

### 4.1 FieldType - 动态模型字段类型

```rust
pub enum FieldType {
    String,      // 短文本
    Richtext,    // 富文本（HTML）
    Image,       // 图片 URL
    File,        // 文件 URL
    Integer,     // 整数
    Number,      // 浮点数
    Boolean,     // 布尔值
    Datetime,    // 日期时间
    Array,       // 数组
    Relation,    // 关联（外键）
}
```

**用途：** 在管理后台动态定义数据模型时，指定字段类型。

**序列化格式：** `snake_case`（如 `richtext`、`datetime`）

```rust
// 字符串解析（大小写不敏感 + 别名支持）
"float".parse::<FieldType>()  → Ok(FieldType::Number)
"bool".parse::<FieldType>()   → Ok(FieldType::Boolean)
"date".parse::<FieldType>()   → Ok(FieldType::Datetime)
```

### 4.2 SystemStatus - 子系统状态

```rust
#[repr(i16)]  // ← 显式指定存储类型
pub enum SystemStatus {
    Disabled = 0,  // 禁用
    Active = 1,    // 激活
    Archived = 2,  // 归档
}

// 从数据库读取时转换
impl SystemStatus {
    pub fn from_i16(val: i16) -> Self {
        match val {
            0 => Self::Disabled,
            1 => Self::Active,
            2 => Self::Archived,
            _ => Self::Disabled,  // ← 默认值（防御性编程）
        }
    }
}
```

### 4.3 is_valid_slug - Slug 格式验证

```rust
pub fn is_valid_slug(slug: &str, max_len: usize) -> bool {
    let len = slug.len();
    
    // 长度检查：2 到 max_len
    if !(2..=max_len).contains(&len) {
        return false;
    }
    
    // 字符检查：只允许小写字母、数字、下划线、连字符
    slug.chars().all(|c| {
        c.is_ascii_lowercase() 
        || c.is_ascii_digit() 
        || c == '_' 
        || c == '-'
    })
}
```

**规则总结：**
- ✅ 允许：`blog`, `vip-mall`, `carnival_2026`
- ❌ 禁止：`Blog`（大写）、`blog!`（特殊字符）、`a`（太短）

**使用场景：**
```rust
// 创建子系统时验证
if !is_valid_slug(&slug, 32) {
    return Err(AppError::Validation(
        "Invalid system slug format".into()
    ));
}

// 创建模型时验证
if !is_valid_slug(&model_slug, 48) {
    return Err(AppError::Validation(
        "Invalid model slug format".into()
    ));
}
```

---

## 5. subsystem.rs - 子系统接口

### 5.1 SubsystemModule Trait

```rust
pub trait SubsystemModule: Send + Sync + 'static {
    fn slug(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn register_routes(&self, router: Router) -> Router;
    
    // 可选方法（有默认实现）
    fn description(&self) -> &'static str { "" }
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> { vec![] }
}
```

### 5.2 Trait 约束解释

```rust
Send + Sync + 'static
```

- **Send**：可以在线程间传递所有权
- **Sync**：可以在多线程间共享引用
- **'static**：生命周期是整个程序运行期间

**原因：** Subsystem 实例会在多个请求处理线程中共享使用。

### 5.3 完整实现示例

```rust
use foundry_core::SubsystemModule;
use axum::{Router, routing::get};

pub struct BlogSubsystem;

impl SubsystemModule for BlogSubsystem {
    fn slug(&self) -> &'static str {
        "blog"  // ← URL 路径：/api/v1/s/blog/...
    }

    fn display_name(&self) -> &'static str {
        "博客系统"  // ← 管理后台显示名称
    }

    fn description(&self) -> &'static str {
        "内容发布与管理系统"
    }

    fn register_routes(&self, router: Router) -> Router {
        router
            .route("/stats", get(handlers::get_stats))
            .route("/trending", get(handlers::get_trending))
        // 路由会被挂载到：/api/v1/s/blog/custom/stats
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![
            CustomAdminPageSpec {
                key: "blog-editor".to_string(),
                title: "文章编辑器".to_string(),
                icon: "edit".to_string(),
                page_type: "react".to_string(),
                entry: "/admin/blog-editor.js".to_string(),
                required_role: Some("admin".to_string()),
            }
        ]
    }
}
```

### 5.4 CustomAdminPageSpec - 自定义管理页面

```rust
pub struct CustomAdminPageSpec {
    pub key: String,              // 唯一标识（如 "blog-editor"）
    pub title: String,             // 页面标题
    pub icon: String,              // 图标名称
    pub page_type: String,         // "react" / "vue" / "html"
    pub entry: String,             // 入口文件路径
    pub required_role: Option<String>,  // 最小权限要求
}
```

**工作原理：**
1. 后端返回所有 CustomAdminPageSpec
2. 前端 Admin 应用动态加载这些页面
3. 页面在 iframe/sandbox 中渲染
4. 通过 postMessage 通信（注入 JWT、主题等）

---

## 6. lib.rs - 模块导出

```rust
pub mod context;
pub mod error;
pub mod response;
pub mod subsystem;
pub mod types;

// 公开导出（用户直接 use foundry_core::SystemContext）
pub use context::SystemContext;
pub use error::{AppError, AppResult, ErrorEnvelope};
pub use response::{ApiResponse, PageMeta, PaginatedData};
pub use subsystem::{CustomAdminPageSpec, SubsystemModule};
pub use types::{FieldType, SystemStatus, is_valid_slug};
```

**设计模式：** Re-export pattern（重导出模式）

**优势：**
```rust
// ❌ 不便：用户需要知道内部模块结构
use foundry_core::error::AppError;
use foundry_core::error::AppResult;

// ✅ 便捷：直接从 crate 根导入
use foundry_core::{AppError, AppResult};
```

---

## 7. 测试覆盖

每个模块都有 `#[cfg(test)] mod tests` 覆盖核心功能：

### context.rs 测试
- ✅ 基础创建和详细创建
- ✅ Redis key 生成
- ✅ 字段正确性

### error.rs 测试
- ✅ 每种错误的状态码映射
- ✅ ErrorEnvelope 生成
- ✅ i18n_key 正确性

### response.rs 测试
- ✅ ApiResponse 构造方法
- ✅ PageMeta 计算（边界情况）
- ✅ PaginatedData 构造

### types.rs 测试
- ✅ Slug 验证（合法/非法字符）
- ✅ FieldType 解析和显示
- ✅ SystemStatus 转换

---

## 8. 关键设计模式总结

### 8.1 类型安全的错误处理

```rust
// ❌ 不好：使用裸 Result
fn get_user() -> Result<User, String> { ... }

// ✅ 更好：使用自定义错误枚举
fn get_user() -> AppResult<User> { ... }
```

### 8.2 泛型响应包装

```rust
// 统一的响应格式，支持任意数据类型
ApiResponse<Article>
ApiResponse<Vec<User>>
ApiResponse<PaginatedData<Product>>
```

### 8.3 Trait 约束明确化

```rust
// 明确要求 Send + Sync，防止编译时错误
trait SubsystemModule: Send + Sync + 'static { ... }
```

### 8.4 防御性编程

```rust
// SystemStatus::from_i16：未知值返回默认值
_ => Self::Disabled,

// PageMeta::new：page_size 为 0 时防止除零
let total_pages = if page_size == 0 { 0 } else { ... };
```

---

## 9. 与其他 Crates 的交互

```
foundry_core (本 crate)
    ↓ 被依赖
┌───────────────────────────────────────┐
│ foundry_storage                       │
│   - 实现 AppResult 返回类型            │
│   - 使用 SystemContext 做租户隔离      │
├───────────────────────────────────────┤
│ foundry_auth                          │
│   - 返回 AppError::Unauthorized       │
│   - 使用 AppResult                     │
├───────────────────────────────────────┤
│ foundry_engine                        │
│   - 实现 SubsystemModule 路由注册     │
│   - 返回 ApiResponse                  │
│   - 使用 AppError 的 IntoResponse     │
├───────────────────────────────────────┤
│ foundry_extension                     │
│   - 使用 SystemContext 传递上下文      │
│   - 返回 AppResult                     │
└───────────────────────────────────────┘
```

---

## 10. 下一步学习建议

理解 `foundry_core` 后，推荐阅读顺序：

1. **foundry_storage** - 看这些类型如何在数据库中使用
2. **foundry_engine/handlers** - 看 AppError 和 ApiResponse 的实际应用
3. **examples/blog_platform** - 看 SubsystemModule 的实战实现

---

## 11. 快速参考

### 常用类型

```rust
use foundry_core::{
    SystemContext,          // 请求上下文
    AppError, AppResult,    // 错误处理
    ApiResponse,            // 成功响应
    PaginatedData,          // 分页数据
    SubsystemModule,        // Subsystem trait
    is_valid_slug,          // Slug 验证
};
```

### 常用模式

```rust
// Handler 函数签名
async fn my_handler(
    Extension(ctx): Extension<SystemContext>,
) -> AppResult<Json<ApiResponse<MyData>>> {
    // ...
}

// 错误处理
let item = store.get(id)
    .await
    .ok_or_else(|| AppError::NotFound("Item not found".into()))?;

// 成功响应
Ok(Json(ApiResponse::success(item)))

// 分页响应
let paginated = PaginatedData::new(items, page, size, total);
Ok(Json(ApiResponse::success(paginated)))
```

---

**✅ 第一阶段完成！** 你现在应该理解了：
- Foundry 的类型系统基础
- 错误处理与响应格式统一
- SubsystemModule 接口设计
- 上下文传递与租户隔离

**准备好进入第二阶段了吗？** 下一步我们将深入 `foundry_storage`，看这些抽象如何在数据库和 Redis 中实现。

---

### 📚 阶段导航
- ⬅️ **上一阶段**：[全景学习指南与代码导读](./)
- 📋 **学习指南总览**：[返回源码深度理解总览](./)
- ➡️ **下一阶段**：[02. 存储层实现与动态模型 (foundry_storage)](./02-storage/)

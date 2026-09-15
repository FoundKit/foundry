---
title: "04. Extension System & Hooks (foundry_extension)"
description: "Stage 4: In-depth analysis of HookContext, ExtensionRegistry, HookRunner, and lifecycle mutation hooks."
---

> **Codebase Navigation**：⬅️ Previous: [03. Authentication & RBAC (foundry_auth)](./03-auth/) · [📋 Overview & Guide](./) · ➡️ Next: [05. Request Engine & Auto-CRUD (foundry_engine)](./05-engine/)

# Foundry Extension 代码深度分析

> 第四阶段：理解扩展系统（生命周期钩子）
>
> Crate: `foundry_extension`
>
> 阅读时间：20-30 分钟

---

## 概述

`foundry_extension` 是 Foundry 框架的**扩展机制层**，提供了：
- **生命周期钩子（Lifecycle Hooks）**
- **钩子管道（Hook Pipeline）**
- **可插拔的扩展点**

**核心理念：** 在不修改框架核心代码的前提下，通过钩子拦截和扩展实体的生命周期事件。

**依赖关系：**
```
foundry_core (基础类型) → foundry_extension (钩子定义)
                            ↓
                        foundry_engine (钩子执行)
```

**关键依赖：**
- `async-trait` - 异步 trait 支持
- `foundry_core` - SystemContext、AppResult
- `serde_json` - JSON 数据操作

---

## 架构总览

```
foundry_extension
├── hooks.rs       # MutationHook trait + HookPipeline
└── lib.rs         # 统一导出
```

**设计模式：**
- **观察者模式（Observer Pattern）**
- **责任链模式（Chain of Responsibility）**
- **插件架构（Plugin Architecture）**

---

## 1. MutationHook Trait - 生命周期钩子接口

### 1.1 Trait 定义

```rust
#[async_trait]
pub trait MutationHook: Send + Sync + 'static {
    // 创建前（可修改数据）
    async fn before_create(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _data: &mut Value,
    ) -> AppResult<()> {
        Ok(())
    }

    // 创建后（只读数据）
    async fn after_create(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        Ok(())
    }

    // 更新前（可修改数据）
    async fn before_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &mut Value,
    ) -> AppResult<()> {
        Ok(())
    }

    // 更新后（只读数据）
    async fn after_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        Ok(())
    }

    // 删除前
    async fn before_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }

    // 删除后
    async fn after_delete(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        Ok(())
    }
}
```

### 1.2 钩子生命周期图

```
CREATE 操作
    ↓
before_create(ctx, model_slug, &mut data)  ← 可修改 data
    ↓
[数据库 INSERT]
    ↓
after_create(ctx, model_slug, record_id, &data)  ← 只读 data
    ↓
完成


UPDATE 操作
    ↓
before_update(ctx, model_slug, id, &mut data)  ← 可修改 data
    ↓
[数据库 UPDATE]
    ↓
after_update(ctx, model_slug, id, &data)  ← 只读 data
    ↓
完成


DELETE 操作
    ↓
before_delete(ctx, model_slug, id)
    ↓
[数据库 DELETE]
    ↓
after_delete(ctx, model_slug, id)
    ↓
完成
```

### 1.3 参数解析

#### SystemContext（上下文）

```rust
_ctx: &SystemContext
```

**提供信息：**
- `system_slug`：当前子系统标识
- `system_name`：子系统名称
- `locale`：用户语言
- `client_ip`：客户端 IP
- `user_agent`：User-Agent

**用途：**
- 租户隔离
- 日志记录
- 审计追踪

#### model_slug（模型标识）

```rust
_model_slug: &str
```

**示例值：** `"articles"`, `"products"`, `"comments"`

**用途：**
- 区分不同模型的钩子逻辑
- 条件执行（只对特定模型生效）

#### data（数据载荷）

```rust
// before 钩子：可变引用
_data: &mut Value

// after 钩子：不可变引用
_data: &Value
```

**类型：** `serde_json::Value`（JSON 对象）

**before 钩子可以修改数据：**
```rust
async fn before_create(
    &self,
    _ctx: &SystemContext,
    _model_slug: &str,
    data: &mut Value,
) -> AppResult<()> {
    if let Some(obj) = data.as_object_mut() {
        // 注入字段
        obj.insert("created_by".to_string(), json!("system"));
        // 修改字段
        if let Some(title) = obj.get_mut("title") {
            *title = json!(title.as_str().unwrap().to_uppercase());
        }
    }
    Ok(())
}
```

#### record_id（记录 ID）

```rust
_record_id: i64
```

**用途：**
- after 钩子中引用新创建的记录
- 关联数据操作
- 审计日志

### 1.4 Trait 约束

```rust
MutationHook: Send + Sync + 'static
```

- **Send**：可以在线程间传递
- **Sync**：可以在多线程间共享引用
- **'static**：生命周期是整个程序运行期间

**原因：** 钩子实例会在多个请求处理线程中并发执行。

### 1.5 默认实现

所有钩子方法都有默认实现（空操作），实现者只需覆盖需要的方法：

```rust
// ✅ 只实现需要的钩子
struct ArticleSlugHook;

#[async_trait]
impl MutationHook for ArticleSlugHook {
    async fn before_create(
        &self,
        _ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if model_slug == "articles" {
            // 自动生成 slug
            if let Some(obj) = data.as_object_mut() {
                if let Some(title) = obj.get("title").and_then(|t| t.as_str()) {
                    obj.insert("slug".to_string(), json!(slugify(title)));
                }
            }
        }
        Ok(())
    }
}
```

---

## 2. HookPipeline - 钩子管道

### 2.1 结构定义

```rust
#[derive(Default, Clone)]
pub struct HookPipeline {
    hooks: Vec<Arc<dyn MutationHook>>,
}
```

**设计要点：**
- `Vec<Arc<...>>`：多个钩子实例
- `Arc`：原子引用计数（线程安全共享）
- `dyn MutationHook`：trait object（动态分发）

### 2.2 注册钩子

```rust
impl HookPipeline {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register<H: MutationHook>(&mut self, hook: H) {
        self.hooks.push(Arc::new(hook));
    }
}
```

**使用示例：**

```rust
let mut pipeline = HookPipeline::new();

// 注册多个钩子
pipeline.register(ArticleSlugHook);
pipeline.register(NotificationHook);
pipeline.register(AuditLogHook);
```

**执行顺序：** 按注册顺序依次执行（FIFO）

### 2.3 执行钩子

#### execute_before_create

```rust
pub async fn execute_before_create(
    &self,
    ctx: &SystemContext,
    model_slug: &str,
    data: &mut Value,
) -> AppResult<()> {
    for hook in &self.hooks {
        hook.before_create(ctx, model_slug, data).await?;
    }
    Ok(())
}
```

**关键点：**
- 依次调用每个钩子的 `before_create`
- 任何一个钩子返回错误 → 立即中断管道
- 数据在钩子间共享（每个钩子都能修改）

#### execute_after_create

```rust
pub async fn execute_after_create(
    &self,
    ctx: &SystemContext,
    model_slug: &str,
    record_id: i64,
    data: &Value,
) -> AppResult<()> {
    for hook in &self.hooks {
        hook.after_create(ctx, model_slug, record_id, data).await?;
    }
    Ok(())
}
```

**关键点：**
- 数据已经持久化到数据库
- `record_id` 已生成
- 数据只读（不能修改）

### 2.4 完整的管道 API

```rust
// before 钩子（可修改数据）
execute_before_create(ctx, model_slug, &mut data) -> AppResult<()>
execute_before_update(ctx, model_slug, id, &mut data) -> AppResult<()>
execute_before_delete(ctx, model_slug, id) -> AppResult<()>

// after 钩子（只读数据）
execute_after_create(ctx, model_slug, id, &data) -> AppResult<()>
execute_after_update(ctx, model_slug, id, &data) -> AppResult<()>
execute_after_delete(ctx, model_slug, id) -> AppResult<()>
```

---

## 3. 实战钩子示例

### 3.1 自动字段注入

```rust
struct AutoFieldsHook;

#[async_trait]
impl MutationHook for AutoFieldsHook {
    async fn before_create(
        &self,
        ctx: &SystemContext,
        _model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if let Some(obj) = data.as_object_mut() {
            // 注入创建时间
            obj.insert(
                "created_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339())
            );
            
            // 注入创建者 IP
            if let Some(ip) = &ctx.client_ip {
                obj.insert("created_ip".to_string(), json!(ip));
            }
        }
        Ok(())
    }

    async fn before_update(
        &self,
        _ctx: &SystemContext,
        _model_slug: &str,
        _record_id: i64,
        data: &mut Value,
    ) -> AppResult<()> {
        if let Some(obj) = data.as_object_mut() {
            // 更新修改时间
            obj.insert(
                "updated_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339())
            );
        }
        Ok(())
    }
}
```

### 3.2 数据验证

```rust
struct BlogValidationHook;

#[async_trait]
impl MutationHook for BlogValidationHook {
    async fn before_create(
        &self,
        _ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        if model_slug == "articles" {
            let obj = data.as_object()
                .ok_or_else(|| AppError::Validation("Data must be object".into()))?;

            // 验证标题长度
            if let Some(title) = obj.get("title").and_then(|t| t.as_str()) {
                if title.len() < 5 {
                    return Err(AppError::Validation(
                        "Article title must be at least 5 characters".into()
                    ));
                }
            }

            // 验证内容不为空
            if let Some(content) = obj.get("content").and_then(|c| c.as_str()) {
                if content.is_empty() {
                    return Err(AppError::Validation(
                        "Article content cannot be empty".into()
                    ));
                }
            }
        }
        Ok(())
    }
}
```

### 3.3 发送通知

```rust
use tokio::spawn;

struct NotificationHook {
    notification_service: Arc<NotificationService>,
}

#[async_trait]
impl MutationHook for NotificationHook {
    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        if model_slug == "articles" {
            let service = self.notification_service.clone();
            let system_slug = ctx.system_slug.clone();
            let data = data.clone();

            // 异步发送通知（不阻塞请求）
            spawn(async move {
                if let Err(e) = service.send_new_article_notification(
                    &system_slug,
                    record_id,
                    &data
                ).await {
                    tracing::error!("Failed to send notification: {}", e);
                }
            });
        }
        Ok(())
    }
}
```

### 3.4 清理关联数据

```rust
struct CascadeDeleteHook {
    db_pool: DbPool,
}

#[async_trait]
impl MutationHook for CascadeDeleteHook {
    async fn before_delete(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
    ) -> AppResult<()> {
        if model_slug == "articles" {
            // 删除文章的所有评论
            sqlx::query(
                "DELETE FROM model_records 
                 WHERE system_id = $1 AND model_slug = 'comments' 
                 AND data->>'article_id' = $2"
            )
            .bind(&ctx.system_slug)
            .bind(record_id.to_string())
            .execute(&self.db_pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }
        Ok(())
    }
}
```

### 3.5 更新缓存

```rust
struct CacheInvalidationHook {
    redis: RedisPool,
}

#[async_trait]
impl MutationHook for CacheInvalidationHook {
    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        _record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        self.invalidate_cache(ctx, model_slug).await
    }

    async fn after_update(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        _data: &Value,
    ) -> AppResult<()> {
        let mut redis = self.redis.clone();
        
        // 删除单条记录缓存
        let key = format!("{}:{}:{}", ctx.system_slug, model_slug, record_id);
        let _: () = redis.del(key).await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 删除列表缓存
        self.invalidate_cache(ctx, model_slug).await
    }

    async fn after_delete(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        _record_id: i64,
    ) -> AppResult<()> {
        self.invalidate_cache(ctx, model_slug).await
    }
}

impl CacheInvalidationHook {
    async fn invalidate_cache(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
    ) -> AppResult<()> {
        let mut redis = self.redis.clone();
        let pattern = format!("{}:{}:*", ctx.system_slug, model_slug);
        
        // 删除所有匹配的键（需要 Lua 脚本或 SCAN）
        let _: () = redis.del(pattern).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        
        Ok(())
    }
}
```

### 3.6 审计日志

```rust
struct AuditLogHook {
    db_pool: DbPool,
}

#[async_trait]
impl MutationHook for AuditLogHook {
    async fn after_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        self.log_action(ctx, "CREATE", model_slug, Some(record_id), data).await
    }

    async fn after_update(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
        data: &Value,
    ) -> AppResult<()> {
        self.log_action(ctx, "UPDATE", model_slug, Some(record_id), data).await
    }

    async fn after_delete(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        record_id: i64,
    ) -> AppResult<()> {
        self.log_action(ctx, "DELETE", model_slug, Some(record_id), &json!({})).await
    }
}

impl AuditLogHook {
    async fn log_action(
        &self,
        ctx: &SystemContext,
        action: &str,
        model_slug: &str,
        record_id: Option<i64>,
        data: &Value,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO hook_audit_logs (system_slug, action, model_slug, record_id, data, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(&ctx.system_slug)
        .bind(action)
        .bind(model_slug)
        .bind(record_id)
        .bind(data)
        .bind(&ctx.client_ip)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
```

---

## 4. 在 Handler 中集成钩子

### 4.1 创建记录

```rust
async fn create_record(
    Extension(pipeline): Extension<Arc<HookPipeline>>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug)): Path<(String, String)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    // 1. 执行 before_create 钩子
    pipeline.execute_before_create(&ctx, &model_slug, &mut payload).await?;

    // 2. 数据库插入
    let record = RecordStore::create(&pool, &system_slug, &model_slug, payload).await?;

    // 3. 执行 after_create 钩子
    pipeline.execute_after_create(&ctx, &model_slug, record.id, &record.data).await?;

    Ok(Json(ApiResponse::success(record)))
}
```

### 4.2 更新记录

```rust
async fn update_record(
    Extension(pipeline): Extension<Arc<HookPipeline>>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
    Json(mut payload): Json<Value>,
) -> AppResult<Json<ApiResponse<ModelRecordEntity>>> {
    // 1. 执行 before_update 钩子
    pipeline.execute_before_update(&ctx, &model_slug, id, &mut payload).await?;

    // 2. 数据库更新
    let record = RecordStore::update(&pool, &system_slug, &model_slug, id, payload).await?;

    // 3. 执行 after_update 钩子
    pipeline.execute_after_update(&ctx, &model_slug, id, &record.data).await?;

    Ok(Json(ApiResponse::success(record)))
}
```

### 4.3 删除记录

```rust
async fn delete_record(
    Extension(pipeline): Extension<Arc<HookPipeline>>,
    Extension(ctx): Extension<SystemContext>,
    Path((system_slug, model_slug, id)): Path<(String, String, i64)>,
) -> AppResult<Json<ApiResponse<()>>> {
    // 1. 执行 before_delete 钩子
    pipeline.execute_before_delete(&ctx, &model_slug, id).await?;

    // 2. 数据库删除
    RecordStore::delete(&pool, &system_slug, &model_slug, id).await?;

    // 3. 执行 after_delete 钩子
    pipeline.execute_after_delete(&ctx, &model_slug, id).await?;

    Ok(Json(ApiResponse::success(())))
}
```

---

## 5. 测试用例

### 5.1 基础管道测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestAuditHook {
        created_count: AtomicUsize,
    }

    #[async_trait]
    impl MutationHook for TestAuditHook {
        async fn before_create(
            &self,
            _ctx: &SystemContext,
            _model_slug: &str,
            data: &mut Value,
        ) -> AppResult<()> {
            // 测试数据修改
            if let Some(obj) = data.as_object_mut() {
                obj.insert("injected_field".to_string(), json!("injected"));
            }
            Ok(())
        }

        async fn after_create(
            &self,
            _ctx: &SystemContext,
            _model_slug: &str,
            _record_id: i64,
            _data: &Value,
        ) -> AppResult<()> {
            // 测试副作用
            self.created_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_hook_pipeline_execution() {
        let mut pipeline = HookPipeline::new();
        let hook = TestAuditHook {
            created_count: AtomicUsize::new(0),
        };
        pipeline.register(hook);

        let ctx = SystemContext::new("test_slug");
        let mut payload = json!({ "name": "original" });

        // 执行 before_create
        pipeline
            .execute_before_create(&ctx, "products", &mut payload)
            .await
            .unwrap();

        // 验证数据被修改
        assert_eq!(payload["injected_field"], "injected");

        // 执行 after_create
        pipeline
            .execute_after_create(&ctx, "products", 101, &payload)
            .await
            .unwrap();

        // 验证副作用生效（这里无法直接访问 hook，需要其他方式验证）
    }
}
```

---

## 6. 关键设计模式总结

### 6.1 观察者模式（Observer Pattern）

```
Subject (RecordStore)
    ↓ 通知
Observer 1 (AutoFieldsHook)
Observer 2 (NotificationHook)
Observer 3 (AuditLogHook)
```

**优势：**
- 解耦：业务逻辑与扩展逻辑分离
- 可插拔：动态添加/移除观察者
- 单一职责：每个钩子负责一个功能

### 6.2 责任链模式（Chain of Responsibility）

```
Request → Hook 1 → Hook 2 → Hook 3 → Response
```

**特点：**
- 按注册顺序依次执行
- 任何一个返回错误 → 中断链
- 数据在钩子间流动

### 6.3 模板方法模式（Template Method）

```rust
// 框架定义骨架
async fn create_record(...) {
    execute_before_create(...);  // ← 扩展点
    database_insert(...);
    execute_after_create(...);   // ← 扩展点
}

// 用户实现具体步骤
impl MutationHook for MyHook {
    async fn before_create(...) {
        // 自定义逻辑
    }
}
```

---

## 7. 最佳实践

### 7.1 钩子命名规范

```rust
// ✅ 推荐：动词 + 名词
AutoFieldsHook
ValidationHook
NotificationHook
AuditLogHook
CacheInvalidationHook

// ❌ 不推荐：含糊不清
MyHook
CustomHook
Hook1
```

### 7.2 钩子职责单一

```rust
// ❌ 不好：一个钩子做太多事
struct MegaHook;
impl MutationHook for MegaHook {
    async fn after_create(...) {
        validate_data(...);     // 应该在 before_create
        send_notification(...); // 应该独立一个钩子
        update_cache(...);      // 应该独立一个钩子
        log_audit(...);         // 应该独立一个钩子
    }
}

// ✅ 更好：每个钩子只做一件事
pipeline.register(ValidationHook);
pipeline.register(NotificationHook);
pipeline.register(CacheInvalidationHook);
pipeline.register(AuditLogHook);
```

### 7.3 异步任务处理

```rust
// ❌ 不好：阻塞请求
async fn after_create(...) {
    send_email(...).await?;  // 耗时操作阻塞响应
}

// ✅ 更好：后台异步执行
async fn after_create(...) {
    let data = data.clone();
    tokio::spawn(async move {
        if let Err(e) = send_email(&data).await {
            tracing::error!("Email failed: {}", e);
        }
    });
    Ok(())
}
```

### 7.4 错误处理

```rust
// 钩子返回错误 → 整个操作回滚
async fn before_create(...) -> AppResult<()> {
    if invalid_data {
        return Err(AppError::Validation("Invalid data".into()));
    }
    Ok(())
}

// 非关键钩子 → 记录日志但不中断
async fn after_create(...) -> AppResult<()> {
    if let Err(e) = send_notification(...).await {
        tracing::error!("Notification failed: {}", e);
        // 不返回错误，继续执行
    }
    Ok(())
}
```

---

## 8. 与其他 Crates 的交互

```
foundry_extension (MutationHook)
    ↓ 被使用
┌───────────────────────────────────────┐
│ foundry_engine (handlers)             │
│   - 在 CRUD 操作中执行钩子            │
├───────────────────────────────────────┤
│ foundry (app.rs)                      │
│   - 初始化 HookPipeline               │
│   - 注册用户自定义钩子                │
├───────────────────────────────────────┤
│ examples/blog_platform (hooks.rs)     │
│   - 实现 BlogMutationHook             │
└───────────────────────────────────────┘
```

---

## 9. 下一步学习建议

理解 `foundry_extension` 后，推荐阅读顺序：

1. **foundry_engine/handlers** - 看钩子如何在 CRUD 中执行
2. **examples/blog_platform/src/hooks.rs** - 看实际钩子实现
3. **foundry/app.rs** - 看钩子如何注册

---

## 10. 快速参考

### 实现钩子

```rust
use foundry_extension::MutationHook;
use async_trait::async_trait;

struct MyHook;

#[async_trait]
impl MutationHook for MyHook {
    async fn before_create(
        &self,
        ctx: &SystemContext,
        model_slug: &str,
        data: &mut Value,
    ) -> AppResult<()> {
        // 自定义逻辑
        Ok(())
    }
}
```

### 注册钩子

```rust
let mut pipeline = HookPipeline::new();
pipeline.register(MyHook);
pipeline.register(AnotherHook);
```

### 执行钩子

```rust
// 创建前
pipeline.execute_before_create(&ctx, "articles", &mut data).await?;

// 创建后
pipeline.execute_after_create(&ctx, "articles", record_id, &data).await?;
```

---

**✅ 第四阶段完成！** 你现在应该理解了：
- MutationHook trait 的设计（6 个生命周期方法）
- HookPipeline 的工作原理（注册、执行、责任链）
- before vs after 钩子的区别（可变 vs 不可变）
- 实战钩子示例（验证、通知、缓存、审计等）
- 设计模式（观察者、责任链、模板方法）
- 最佳实践（职责单一、异步处理、错误处理）

**准备好进入第五阶段了吗？** 下一步我们将深入 `foundry_engine`，这是最复杂的 crate，包含路由、处理器、中间件和 Auto-CRUD 实现。
---

### 📚 Stage Navigation
- ⬅️ **Previous Stage**: [03. Authentication & RBAC (foundry_auth)](./03-auth/)
- 📋 **Guide Overview**: [Return to Overview](./)
- ➡️ **Next Stage**: [05. Request Engine & Auto-CRUD (foundry_engine)](./05-engine/)

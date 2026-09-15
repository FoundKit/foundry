---
title: "03. 认证与权限控制系统 (foundry_auth)"
description: "阶段 3：深度剖析管理员模型、密码哈希、JWT 令牌流转与 RBAC 权限判定。"
---

> **源码导读导航**：⬅️ 上一篇：[02. 存储层实现与动态模型 (foundry_storage)](./02-storage/) · [📋 学习指南与总览](./) · ➡️ 下一篇：[04. 扩展系统与生命周期钩子 (foundry_extension)](./04-extension/)

# Foundry Auth 代码深度分析

> 第三阶段：理解认证与授权实现
>
> Crate: `foundry_auth`
>
> 阅读时间：30-45 分钟

---

## 概述

`foundry_auth` 是 Foundry 框架的**安全层**，实现了：
- **密码哈希与验证**（Argon2id）
- **JWT 令牌生成与验证**
- **基于角色的访问控制（RBAC）**

**依赖关系：**
```
foundry_core (基础类型) → foundry_auth (认证)
foundry_storage (数据访问) → foundry_auth (查询管理员)
```

**关键依赖：**
- `argon2` - Argon2id 密码哈希算法（OWASP 推荐）
- `jsonwebtoken` - JWT 生成和验证
- `uuid` - 管理员唯一标识
- `chrono` - 令牌过期时间计算

---

## 架构总览

```
foundry_auth
├── password.rs    # 密码哈希与验证（Argon2id）
├── jwt.rs         # JWT 令牌服务 + AdminClaims
├── rbac.rs        # 基于角色的访问控制
└── lib.rs         # 统一导出
```

**安全设计理念：**
1. **零明文密码**：所有密码都经过 Argon2id 哈希存储
2. **无状态认证**：JWT 自包含所有权限信息
3. **最小权限原则**：Topic Admin 只能访问分配的子系统
4. **细粒度 RBAC**：三级角色体系（Super Admin > Admin > Topic Admin）

---

## 1. password.rs - 密码安全

### 1.1 Argon2id 算法

**为什么选择 Argon2id？**
- 2015 年密码哈希竞赛（PHC）冠军
- OWASP 推荐的密码哈希算法
- 抗 GPU/ASIC 暴力破解（内存困难型）
- 结合了 Argon2i（抗旁路攻击）和 Argon2d（抗 GPU 攻击）的优势

**对比其他算法：**

| 算法 | 安全性 | 速度 | OWASP 推荐 |
|------|--------|------|-----------|
| MD5 | ❌ 已破解 | 极快 | ❌ 禁用 |
| SHA-256 | ⚠️ 不够慢 | 很快 | ❌ 不推荐 |
| bcrypt | ✅ 安全 | 较慢 | ✅ 可用 |
| scrypt | ✅ 安全 | 慢 | ✅ 可用 |
| **Argon2id** | ✅ 最安全 | 可配置 | ✅ **首选** |

### 1.2 密码哈希实现

```rust
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);  // ← 加密安全的随机盐
    let argon2 = Argon2::default();               // ← 使用默认参数
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();

    Ok(hash)
}
```

**关键步骤：**

1. **生成随机盐（Salt）**
   ```rust
   let salt = SaltString::generate(&mut OsRng);
   ```
   - `OsRng`：使用操作系统的加密随机数生成器
   - 每个密码使用不同的盐 → 防止彩虹表攻击

2. **哈希密码**
   ```rust
   argon2.hash_password(password.as_bytes(), &salt)
   ```
   - 输入：明文密码 + 盐
   - 输出：PHC 格式字符串

3. **PHC 格式输出**
   ```
   $argon2id$v=19$m=19456,t=2,p=1$saltbase64$hashbase64
   ```
   - `argon2id`：算法标识
   - `v=19`：版本号
   - `m=19456`：内存成本（KiB）
   - `t=2`：时间成本（迭代次数）
   - `p=1`：并行度
   - `saltbase64`：盐的 Base64 编码
   - `hashbase64`：哈希值的 Base64 编码

**默认参数分析：**
```rust
Argon2::default()
```
对应配置：
- **内存成本（m）**：19456 KiB ≈ 19 MB
- **时间成本（t）**：2 次迭代
- **并行度（p）**：1 个线程

**性能权衡：**
- 哈希一个密码耗时：约 50-100ms（可接受）
- 暴力破解：每秒仅能尝试 10-20 个密码（有效防护）

### 1.3 密码验证实现

```rust
pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;

    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

**验证流程：**

1. **解析存储的哈希**
   ```rust
   PasswordHash::new(password_hash)
   ```
   - 从 PHC 格式字符串中提取：算法、参数、盐、哈希值

2. **重新哈希输入密码**
   ```rust
   argon2.verify_password(password.as_bytes(), &parsed_hash)
   ```
   - 使用相同的盐和参数
   - 重新计算哈希

3. **常量时间比较**
   - 防止时间侧信道攻击
   - 无论密码正确与否，比较时间相同

### 1.4 测试用例

```rust
#[test]
fn test_password_hash_and_verify() {
    let password = "admin123456";
    
    // 哈希密码
    let hash = hash_password(password).expect("Hashing should succeed");
    
    // 验证正确密码
    assert!(verify_password(password, &hash).expect("Verification should succeed"));
    
    // 验证错误密码
    assert!(!verify_password("wrong_password", &hash).expect("Verification should succeed"));
}
```

**注意：** 每次哈希同一密码都会得到不同的结果（因为盐不同），但都能通过验证。

```rust
let hash1 = hash_password("admin123456").unwrap();
let hash2 = hash_password("admin123456").unwrap();
assert_ne!(hash1, hash2);  // ← 哈希值不同
assert!(verify_password("admin123456", &hash1).unwrap());  // ← 但都能验证
assert!(verify_password("admin123456", &hash2).unwrap());
```

---

## 2. jwt.rs - JWT 令牌服务

### 2.1 AdminClaims - JWT 载荷

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: Uuid,                  // Subject：管理员 ID
    pub username: String,           // 用户名
    pub role: String,               // 角色：super_admin | admin | topic_admin
    pub allowed_systems: Vec<String>, // 允许访问的子系统列表
    pub exp: usize,                 // Expiration：过期时间戳
    pub iat: usize,                 // Issued At：签发时间戳
}
```

**JWT 标准字段：**
- `sub`（Subject）：令牌的主体（用户标识）
- `exp`（Expiration Time）：过期时间
- `iat`（Issued At）：签发时间

**自定义字段：**
- `username`：便于日志记录和审计
- `role`：角色标识（RBAC 核心）
- `allowed_systems`：细粒度权限控制

### 2.2 角色判断方法

```rust
impl AdminClaims {
    pub fn is_super_admin(&self) -> bool {
        self.role == "super_admin"
    }

    pub fn is_general_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_topic_admin(&self) -> bool {
        self.role == "topic_admin"
    }

    /// 是否有平台级管理权限
    pub fn has_platform_manage_access(&self) -> bool {
        self.role == "super_admin"
            || self.role == "admin"
            || self.allowed_systems.iter().any(|s| s == "*")
    }

    /// 是否能管理管理员
    pub fn can_manage_admins(&self) -> bool {
        self.role == "super_admin"
    }

    /// 是否能查看平台统计
    pub fn can_view_platform_summary(&self) -> bool {
        self.role == "super_admin" || self.role == "admin"
    }
}
```

**权限矩阵：**

| 权限 | Super Admin | Admin | Topic Admin |
|------|-------------|-------|-------------|
| 管理所有子系统 | ✅ | ✅ | ❌（仅分配的）|
| 管理管理员 | ✅ | ❌ | ❌ |
| 查看平台统计 | ✅ | ✅ | ❌ |
| 修改系统配置 | ✅ | ✅ | ✅（仅分配的）|
| 查看审计日志 | ✅ | ✅ | ✅（仅分配的）|

### 2.3 JwtService - 令牌服务

```rust
pub struct JwtService {
    secret: String,      // JWT 签名密钥
    expire_hours: i64,   // 令牌有效期（小时）
}

impl JwtService {
    pub fn new(secret: impl Into<String>, expire_hours: i64) -> Self {
        Self {
            secret: secret.into(),
            expire_hours,
        }
    }
}
```

**配置建议：**
- `secret`：至少 32 字节的随机字符串（环境变量 `JWT_SECRET`）
- `expire_hours`：
  - 开发环境：24 小时
  - 生产环境：2-8 小时（提高安全性）

### 2.4 生成令牌

```rust
pub fn generate_token(
    &self,
    admin_id: Uuid,
    username: &str,
    role: &str,
    allowed_systems: Vec<String>,
) -> AppResult<String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(self.expire_hours)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = AdminClaims {
        sub: admin_id,
        username: username.to_string(),
        role: role.to_string(),
        allowed_systems,
        exp,
        iat,
    };

    encode(
        &Header::default(),  // ← 使用默认 Header（HS256 算法）
        &claims,
        &EncodingKey::from_secret(self.secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate JWT: {}", e)))
}
```

**JWT 结构：**
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1ZjY3...Uw.SflKxwRJ...MeJf36
|____________Header_____________|.|________Payload________|.|_Signature_|
```

- **Header**：`{"alg": "HS256", "typ": "JWT"}`
- **Payload**：Base64(AdminClaims)
- **Signature**：HMAC-SHA256(Header + Payload, secret)

**签名算法：**
```
HMAC-SHA256(
    base64UrlEncode(header) + "." + base64UrlEncode(payload),
    secret
)
```

**安全性：**
- 签名保证令牌未被篡改
- 密钥泄露 → 所有令牌失效 → 需要立即更换密钥
- 令牌过期 → 自动失效

### 2.5 验证令牌

```rust
pub fn verify_token(&self, token: &str) -> AppResult<AdminClaims> {
    let validation = Validation::default();  // ← 默认验证规则
    let token_data = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(self.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {}", e)))?;

    Ok(token_data.claims)
}
```

**验证步骤：**

1. **解析 JWT 结构**
   - 分割为 Header、Payload、Signature

2. **验证签名**
   - 重新计算签名
   - 与令牌中的签名比对

3. **验证过期时间**
   ```rust
   if claims.exp < current_timestamp {
       return Err("Token expired");
   }
   ```

4. **返回 Claims**
   - 验证通过 → 返回 `AdminClaims`
   - 验证失败 → 返回 `AppError::Unauthorized`

**常见错误：**
- `InvalidToken`：格式错误
- `InvalidSignature`：签名验证失败（令牌被篡改）
- `ExpiredSignature`：令牌已过期

### 2.6 测试用例

```rust
#[test]
fn test_jwt_generation_and_validation() {
    let jwt = JwtService::new("secret_test_key_1234567890", 24);
    let admin_id = Uuid::new_v4();
    
    // 生成令牌
    let token = jwt
        .generate_token(admin_id, "admin", "super_admin", vec!["*".to_string()])
        .unwrap();

    // 验证令牌
    let claims = jwt.verify_token(&token).unwrap();
    assert_eq!(claims.sub, admin_id);
    assert_eq!(claims.username, "admin");
    assert_eq!(claims.role, "super_admin");
    assert_eq!(claims.allowed_systems, vec!["*"]);
}
```

---

## 3. rbac.rs - 基于角色的访问控制

### 3.1 check_system_access - 核心授权函数

```rust
pub fn check_system_access(
    claims: &AdminClaims,
    target_system_slug: &str
) -> AppResult<()> {
    // 1. Super Admin 和 General Admin 有全局权限
    if claims.role == "super_admin" || claims.role == "admin" {
        return Ok(());
    }

    // 2. Topic Admin 检查白名单
    if claims
        .allowed_systems
        .iter()
        .any(|s| s == "*" || s == target_system_slug)
    {
        return Ok(());
    }

    // 3. 拒绝访问
    Err(AppError::Forbidden(format!(
        "Administrator '{}' is not authorized to manage sub-system '{}'",
        claims.username, target_system_slug
    )))
}
```

**授权逻辑图：**

```
请求访问 system_slug
    ↓
┌────────────────────────────────────┐
│ claims.role == "super_admin"?      │ → ✅ 允许
└────────────────────────────────────┘
    ↓ 否
┌────────────────────────────────────┐
│ claims.role == "admin"?            │ → ✅ 允许
└────────────────────────────────────┘
    ↓ 否
┌────────────────────────────────────┐
│ "*" in allowed_systems?            │ → ✅ 允许
└────────────────────────────────────┘
    ↓ 否
┌────────────────────────────────────┐
│ target_slug in allowed_systems?    │ → ✅ 允许
└────────────────────────────────────┘
    ↓ 否
❌ Forbidden
```

### 3.2 使用场景

#### 场景 1：Handler 中权限检查

```rust
async fn create_article(
    Extension(claims): Extension<AdminClaims>,
    Path(system_slug): Path<String>,
    Json(payload): Json<CreateArticleDto>,
) -> AppResult<Json<ApiResponse<Article>>> {
    // 1. 检查是否有权限访问此子系统
    check_system_access(&claims, &system_slug)?;

    // 2. 执行业务逻辑
    let article = ArticleService::create(&system_slug, payload).await?;

    Ok(Json(ApiResponse::success(article)))
}
```

#### 场景 2：中间件中全局检查

```rust
async fn rbac_middleware(
    Extension(claims): Extension<AdminClaims>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 从路径参数提取 system_slug
    if let Some(system_slug) = params.get("system_slug") {
        check_system_access(&claims, system_slug)?;
    }

    Ok(next.run(request).await)
}
```

### 3.3 测试用例

#### 测试 1：Super Admin 全局访问

```rust
#[test]
fn test_super_admin_access() {
    let claims = AdminClaims {
        sub: Uuid::new_v4(),
        username: "super_boss".to_string(),
        role: "super_admin".to_string(),
        allowed_systems: vec!["*".to_string()],
        exp: 9999999999,
        iat: 1000000000,
    };

    // Super Admin 可以访问任何系统
    assert!(check_system_access(&claims, "carnival_2026").is_ok());
    assert!(check_system_access(&claims, "vip_mall").is_ok());
    assert!(check_system_access(&claims, "any_system").is_ok());

    // 权限方法测试
    assert!(claims.is_super_admin());
    assert!(claims.can_manage_admins());
    assert!(claims.has_platform_manage_access());
    assert!(claims.can_view_platform_summary());
}
```

#### 测试 2：General Admin 全局访问（但不能管理管理员）

```rust
#[test]
fn test_general_admin_access() {
    let claims = AdminClaims {
        sub: Uuid::new_v4(),
        username: "general_op".to_string(),
        role: "admin".to_string(),
        allowed_systems: vec!["*".to_string()],
        exp: 9999999999,
        iat: 1000000000,
    };

    assert!(claims.is_general_admin());
    assert!(!claims.can_manage_admins());  // ← 不能管理管理员
    assert!(claims.has_platform_manage_access());
    assert!(claims.can_view_platform_summary());
    
    // 可以访问所有系统
    assert!(check_system_access(&claims, "carnival_2026").is_ok());
    assert!(check_system_access(&claims, "vip_mall").is_ok());
}
```

#### 测试 3：Topic Admin 受限访问

```rust
#[test]
fn test_topic_admin_access() {
    let claims = AdminClaims {
        sub: Uuid::new_v4(),
        username: "carnival_manager".to_string(),
        role: "topic_admin".to_string(),
        allowed_systems: vec!["carnival_2026".to_string()],  // ← 只能访问这一个
        exp: 9999999999,
        iat: 1000000000,
    };

    assert!(claims.is_topic_admin());
    assert!(!claims.can_manage_admins());
    assert!(!claims.can_view_platform_summary());
    
    // 只能访问分配的系统
    assert!(check_system_access(&claims, "carnival_2026").is_ok());
    assert!(check_system_access(&claims, "vip_mall").is_err());  // ← 拒绝访问
}
```

---

## 4. 完整认证流程

### 4.1 注册流程

```rust
async fn register_admin(
    pool: &DbPool,
    username: &str,
    password: &str,
    role: &str,
    allowed_systems: Vec<String>,
) -> AppResult<AdminEntity> {
    // 1. 哈希密码
    let password_hash = hash_password(password)?;

    // 2. 存储到数据库
    let admin = AdminStore::create(
        pool,
        username,
        None,  // email
        &password_hash,
        role,
        serde_json::to_value(allowed_systems).unwrap(),
    ).await?;

    Ok(admin)
}
```

### 4.2 登录流程

```rust
async fn login(
    pool: &DbPool,
    jwt_service: &JwtService,
    username: &str,
    password: &str,
) -> AppResult<String> {
    // 1. 查询管理员
    let admin = AdminStore::get_by_username(pool, username).await?;

    // 2. 验证密码
    if !verify_password(password, &admin.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    // 3. 检查账户状态
    if admin.status != 1 {
        return Err(AppError::Forbidden("Account disabled".into()));
    }

    // 4. 解析 allowed_systems
    let allowed_systems: Vec<String> = serde_json::from_value(admin.allowed_systems)
        .unwrap_or_default();

    // 5. 生成 JWT
    let token = jwt_service.generate_token(
        admin.id,
        &admin.username,
        &admin.role,
        allowed_systems,
    )?;

    Ok(token)
}
```

### 4.3 请求认证流程

```rust
// Axum 中间件
async fn auth_middleware(
    jwt_service: Extension<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. 提取 Authorization Header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    // 2. 提取 Token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format".into()))?;

    // 3. 验证 Token
    let claims = jwt_service.verify_token(token)?;

    // 4. 注入到请求扩展
    request.extensions_mut().insert(claims);

    // 5. 继续处理请求
    Ok(next.run(request).await)
}

// Handler 中使用
async fn protected_handler(
    Extension(claims): Extension<AdminClaims>,
) -> impl IntoResponse {
    format!("Hello, {}! Role: {}", claims.username, claims.role)
}
```

---

## 5. 安全最佳实践

### 5.1 密码策略

**最低要求：**
```rust
fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".into()));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation("Password must contain at least one digit".into()));
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(AppError::Validation("Password must contain at least one lowercase letter".into()));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(AppError::Validation("Password must contain at least one uppercase letter".into()));
    }
    Ok(())
}
```

### 5.2 JWT Secret 管理

**生成强密钥：**
```bash
# Linux/Mac
openssl rand -base64 32

# 输出示例：
# j8K9mN2pQ3rS4tU5vW6xY7zA8bC9dE0fF1gG2hH3iI4j
```

**环境变量配置：**
```bash
# .env
JWT_SECRET=j8K9mN2pQ3rS4tU5vW6xY7zA8bC9dE0fF1gG2hH3iI4j
JWT_EXPIRE_HOURS=8
```

**注意事项：**
- ❌ 不要硬编码在代码中
- ❌ 不要提交到 Git
- ✅ 使用环境变量
- ✅ 不同环境使用不同密钥
- ✅ 定期轮换密钥

### 5.3 令牌刷新策略

```rust
// 短期 Access Token + 长期 Refresh Token
pub struct TokenPair {
    pub access_token: String,   // 1小时过期
    pub refresh_token: String,  // 7天过期
}

impl JwtService {
    pub fn generate_token_pair(&self, admin: &AdminEntity) -> AppResult<TokenPair> {
        // Access Token：短期，包含完整权限
        let access_token = self.generate_token(
            admin.id,
            &admin.username,
            &admin.role,
            parse_allowed_systems(&admin.allowed_systems),
        )?;

        // Refresh Token：长期，仅包含用户 ID
        let refresh_token = self.generate_refresh_token(admin.id)?;

        Ok(TokenPair { access_token, refresh_token })
    }

    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        pool: &DbPool,
    ) -> AppResult<String> {
        // 1. 验证 Refresh Token
        let claims = self.verify_token(refresh_token)?;

        // 2. 重新查询管理员信息（防止权限变更）
        let admin = AdminStore::get_by_id(pool, claims.sub).await?;

        // 3. 生成新的 Access Token
        self.generate_token(
            admin.id,
            &admin.username,
            &admin.role,
            parse_allowed_systems(&admin.allowed_systems),
        )
    }
}
```

### 5.4 防止暴力破解

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window_secs: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check_rate_limit(&self, username: &str) -> AppResult<()> {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();

        let entry = attempts.entry(username.to_string()).or_insert_with(Vec::new);

        // 清理过期记录
        entry.retain(|&t| now.duration_since(t) < self.window);

        // 检查限流
        if entry.len() >= self.max_attempts {
            return Err(AppError::Forbidden(format!(
                "Too many login attempts. Try again in {} seconds",
                self.window.as_secs()
            )));
        }

        // 记录本次尝试
        entry.push(now);
        Ok(())
    }
}

// 使用示例：
// 5 分钟内最多 5 次登录尝试
let rate_limiter = RateLimiter::new(5, 300);

async fn login_with_rate_limit(
    rate_limiter: &RateLimiter,
    username: &str,
    password: &str,
) -> AppResult<String> {
    // 检查限流
    rate_limiter.check_rate_limit(username)?;

    // 执行登录
    login(pool, jwt_service, username, password).await
}
```

### 5.5 HTTPS 强制

```rust
// Axum 中间件强制 HTTPS
async fn force_https_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().scheme_str() != Some("https") {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}
```

---

## 6. 关键设计模式总结

### 6.1 无状态认证（Stateless Authentication）

**传统 Session：**
```
服务器存储 Session
  ↓
需要 Redis/Memcached
  ↓
水平扩展困难
```

**JWT 令牌：**
```
Token 自包含所有信息
  ↓
服务器无状态
  ↓
易于水平扩展
```

### 6.2 最小权限原则（Principle of Least Privilege）

```rust
// ❌ 不好：所有管理员都是全局权限
if claims.role == "admin" {
    // 允许访问任何系统
}

// ✅ 更好：细粒度权限控制
if claims.role == "super_admin" || claims.role == "admin" {
    // 全局权限
} else if claims.allowed_systems.contains(target_slug) {
    // 限定权限
} else {
    // 拒绝
}
```

### 6.3 加盐哈希（Salted Hash）

```rust
// ❌ 不好：不加盐
hash = SHA256(password)

// ✅ 更好：随机盐
salt = random_bytes(16)
hash = Argon2id(password, salt)
```

**防止：** 彩虹表攻击

---

## 7. 与其他 Crates 的交互

```
foundry_auth
    ↓ 被使用
┌───────────────────────────────────────┐
│ foundry_engine (middleware)           │
│   - 使用 JwtService 验证令牌          │
│   - 使用 check_system_access 鉴权     │
├───────────────────────────────────────┤
│ foundry_engine (handlers/auth.rs)     │
│   - 使用 hash_password 注册           │
│   - 使用 verify_password 登录         │
│   - 使用 JwtService::generate_token   │
├───────────────────────────────────────┤
│ foundry (app.rs)                      │
│   - 初始化 JwtService                 │
└───────────────────────────────────────┘
```

---

## 8. 测试覆盖

### 8.1 单元测试总结

```rust
// password.rs
#[test] fn test_password_hash_and_verify()

// jwt.rs
#[test] fn test_jwt_generation_and_validation()

// rbac.rs
#[test] fn test_super_admin_access()
#[test] fn test_general_admin_access()
#[test] fn test_topic_admin_access()
```

### 8.2 集成测试建议

```rust
#[tokio::test]
async fn test_full_auth_flow() {
    let pool = setup_test_db().await;
    let jwt = JwtService::new("test_secret", 24);

    // 1. 注册
    let password_hash = hash_password("admin123").unwrap();
    let admin = AdminStore::create(
        &pool,
        "testuser",
        None,
        &password_hash,
        "admin",
        json!([]),
    ).await.unwrap();

    // 2. 登录
    assert!(verify_password("admin123", &admin.password_hash).unwrap());

    // 3. 生成令牌
    let token = jwt.generate_token(
        admin.id,
        &admin.username,
        &admin.role,
        vec![],
    ).unwrap();

    // 4. 验证令牌
    let claims = jwt.verify_token(&token).unwrap();
    assert_eq!(claims.sub, admin.id);
}
```

---

## 9. 常见问题

### Q1: 为什么不用 bcrypt？

**A:** Argon2id 更现代、更安全，是 OWASP 的首选推荐。bcrypt 虽然也安全，但：
- 内存成本固定（易受 GPU 攻击）
- 算法较老（2019 年）
- Argon2id 是 2015 年密码哈希竞赛冠军

### Q2: JWT Secret 多长才安全？

**A:** 至少 32 字节（256 位）。推荐使用 `openssl rand -base64 32` 生成。

### Q3: 令牌过期后如何处理？

**A:** 两种方案：
1. **简单方案**：令牌过期 → 重新登录
2. **优化方案**：Access Token（短期） + Refresh Token（长期） → 自动刷新

### Q4: 如何撤销 JWT？

**A:** JWT 是无状态的，无法直接撤销。解决方案：
1. **短过期时间**：令牌很快自动失效
2. **黑名单机制**：在 Redis 中记录已撤销的令牌 ID
3. **版本号机制**：令牌中包含版本号，修改密码时增加版本号

---

## 10. 下一步学习建议

理解 `foundry_auth` 后，推荐阅读顺序：

1. **foundry_engine/middleware** - 看认证中间件如何集成
2. **foundry_engine/handlers/auth.rs** - 看登录/注册 API 实现
3. **examples/blog_platform** - 看实际项目中的认证使用

---

## 11. 快速参考

### 常用操作

```rust
use foundry_auth::{hash_password, verify_password, JwtService, check_system_access};

// 密码哈希
let hash = hash_password("password123")?;
let is_valid = verify_password("password123", &hash)?;

// JWT 服务
let jwt = JwtService::new("your-secret-key", 24);
let token = jwt.generate_token(admin_id, "admin", "super_admin", vec![])?;
let claims = jwt.verify_token(&token)?;

// 权限检查
check_system_access(&claims, "blog")?;
```

---

**✅ 第三阶段完成！** 你现在应该理解了：
- Argon2id 密码哈希的原理和优势
- JWT 令牌的生成、验证和结构
- AdminClaims 的设计和角色判断
- 三级 RBAC 体系（Super Admin、Admin、Topic Admin）
- 完整的认证授权流程
- 安全最佳实践

**准备好进入第四阶段了吗？** 下一步我们将深入 `foundry_extension`，看生命周期钩子系统如何工作。

---

### 📚 阶段导航
- ⬅️ **上一阶段**：[02. 存储层实现与动态模型 (foundry_storage)](./02-storage/)
- 📋 **学习指南总览**：[返回源码深度理解总览](./)
- ➡️ **下一阶段**：[04. 扩展系统与生命周期钩子 (foundry_extension)](./04-extension/)

---
title: 应用配置与环境变量完全指南
description: Foundry 全局运行时配置与环境系统完全参考：包含环境变量速查表、FoundryBuilder 流式 API、数据库连接池调优与运行时动态业务配置。
---

Foundry 秉持 **“约定优于配置，极简且高可控”** 的设计哲学。系统将配置清晰地划分为两层：
1. **静态基础设施配置 (Static Infrastructure Config)**：通过环境变量、`.env` 文件以及 `FoundryBuilder` 链式方法配置主机监听、数据库连接串、Redis、JWT 密钥与邮件中继；
2. **运行时动态业务配置 (Runtime Dynamic Business Config)**：通过 `ConfigSchema` 驱动的动态配置中心，在管理控制台可视化定义并在业务运行期间实时修改、热生效，无需重启服务。

---

## 1. 环境变量完全参考清单

在本地开发时，Foundry 会通过 `dotenvy` 自动读取项目根目录下的 `.env` 文件。在 Docker、Kubernetes 或生产服务器部署时，则优先读取系统环境变量。

以下是 Foundry 平台支持的完整环境变量列表：

| 环境变量名 | 类型 | 默认值 | 说明与示例 |
|---|---|---|---|
| `HOST` | String | `0.0.0.0` | HTTP 服务监听的本地网卡 IP 地址。本地测试可设为 `127.0.0.1`，容器内应保持 `0.0.0.0`。 |
| `PORT` | u16 | `8080` | HTTP 服务绑定的监听端口。 |
| `DATABASE_URL` | String | `postgres://postgres:postgrespassword@localhost:5432/foundry` | 底层数据库连接 URL。系统根据协议前缀（`postgres://`、`mysql://`、`mariadb://`、`mongodb://`）自动激活对应 SPI 驱动。 |
| `DATABASE_TYPE` | String | *(空 / 自动探测)* | 显式指定数据库方言类型（`postgres`、`mysql`、`mariadb`、`mongodb`），优先级高于 URL 前缀。 |
| `DATABASE_SSL_MODE` | String | `preferred` | 数据库 SSL/TLS 握手模式（支持 `disabled`、`preferred`、`required`、`verify_ca`、`verify_identity`）。在内网自签名证书场景下可设为 `disabled`。 |
| `REDIS_URL` | String | *(空 / 可选)* | Redis 缓存服务连接 URL（如 `redis://127.0.0.1:6379`）。若不配置，系统将优雅降级为无缓存模式运行。 |
| `JWT_SECRET` | String | `foundry_super_secret_jwt_key_2026_change_in_production` | 管理员 JWT Token 签名秘钥。**⚠️ 生产环境中必须设置为高强度随机字符串！** |
| `AUTO_MIGRATE` | bool | `true` | 服务启动时是否自动执行基线数据库初始化与增量迁移（支持 `true`、`1`、`false`、`0`）。 |
| `SMTP_HOST` | String | *(空 / 可选)* | SMTP 邮件服务器主机名（如 `smtp.example.com`）。配置后将自动启用管理员邮箱验证码登录与绑定服务。 |
| `SMTP_PORT` | u16 | `587` | SMTP 邮件服务器端口（常用端口：`587` TLS、`465` SSL、`25` 普通）。 |
| `SMTP_USER` | String | *(空 / 可选)* | SMTP 认证用户名（通常为发信邮箱账号）。 |
| `SMTP_PASS` | String | *(空 / 可选)* | SMTP 认证密码或授权应用专用访问密码（App Password）。 |
| `SMTP_FROM` | String | `Foundry <no-reply@foundry.local>` | 邮件发件人展示名称与信箱。 |
| `RUST_LOG` | String | `info,foundry=debug,foundry_engine=debug` | 日志过滤等级规则，详见 [日志指南](./logging/)。 |

---

## 2. `.env` 配置文件模板

在应用根目录下创建 `.env` 文件（由脚手架生成或参考 `.env.example`）：

```bash
# ==============================================================================
# FOUNDRY 平台核心运行配置
# ==============================================================================

# 1. 服务监听
HOST=127.0.0.1
PORT=8080

# 2. 数据库连接串 (根据实际所用数据库按需选择)
# PostgreSQL:
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
# MySQL (8.0+):
# DATABASE_URL=mysql://root:root@localhost:3306/foundry
# MariaDB (10.5+):
# DATABASE_URL=mariadb://root:root@localhost:3307/foundry
# MongoDB (6.0+):
# DATABASE_URL=mongodb://127.0.0.1:27017/foundry
# 可选 SSL/TLS 模式 (disabled | preferred | required | verify_ca | verify_identity)
# DATABASE_SSL_MODE=preferred

# 3. 自动迁移开关 (首次运行建议开启自动建表)
AUTO_MIGRATE=true

# 4. 可选 Redis 缓存服务
REDIS_URL=redis://127.0.0.1:6379

# 5. JWT 安全秘钥
JWT_SECRET=super_secret_jwt_key_for_my_production_app_2026

# 6. 可选 SMTP 邮件发送配置 (用于邮箱登录/绑定验证码)
# SMTP_HOST=smtp.gmail.com
# SMTP_PORT=587
# SMTP_USER=notifications@my-domain.com
# SMTP_PASS=your_app_specific_password
# SMTP_FROM=Foundry Platform <notifications@my-domain.com>

# 7. 日志输出过滤
RUST_LOG=info,my_app=debug,foundry=debug,foundry_engine=debug
```

---

## 3. 代码级流式配置 (`FoundryBuilder`)

除了环境变量之外，你还可以在代码中使用 `FoundryApp::builder()` 提供的方法进行强类型覆写与编程化配置：

```rust
use foundry::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 从环境变量加载基准配置
    let base_config = FoundryConfig::from_env();

    // 2. 使用流式 Builder 组装与覆盖配置
    let app = FoundryApp::builder()
        .config(base_config)
        // 覆盖监听地址与端口
        .host("0.0.0.0")
        .port(8080)
        // 数据库连接池大小（默认 20）
        .db_pool_size(50)
        // 自定义 JWT 过期时间（例如 72 小时）
        .jwt_expiry_hours(72)
        // 自动迁移设置
        .auto_migrate(true)
        // 挂载外部子系统目录
        .with_external_systems_dir(PathBuf::from("./external_systems"))
        // 挂载外部 Admin 前端静态文件构建产物
        .with_admin_spa_dir(PathBuf::from("./static/admin"))
        // 注册业务子系统与突变钩子
        .register_subsystem(crate::systems::ShopSubsystem)
        // 构建应用实例
        .build()
        .await?;

    // 3. 启动并监听服务
    app.run().await?;
    Ok(())
}
```

### 3.1 常用 Builder 方法一览

| 方法 | 参数类型 | 默认行为 | 说明 |
|---|---|---|---|
| `config(FoundryConfig)` | `FoundryConfig` | `FoundryConfig::default()` | 批量设置基础结构体配置。 |
| `host(impl Into<String>)` | `&str` / `String` | 读取 `HOST` 或 `0.0.0.0` | 设置 HTTP 绑定 IP。 |
| `port(u16)` | `u16` | 读取 `PORT` 或 `8080` | 设置 HTTP 监听端口。 |
| `database_url(...)` | `&str` / `String` | 读取 `DATABASE_URL` | 设置数据库 URL。 |
| `redis_url(Option<String>)` | `Option<String>` | 读取 `REDIS_URL` | 设置 Redis 缓存地址。 |
| `jwt_secret(...)` | `&str` / `String` | 读取 `JWT_SECRET` | 设置 JWT 签名私钥。 |
| `jwt_expiry_hours(i64)` | `i64` | `168` (7 天) | 设置生成的 JWT 令牌有效时长（小时）。 |
| `auto_migrate(bool)` | `bool` | `true` | 是否在启动时检查并执行数据库迁移。 |
| `db_pool_size(u32)` | `u32` | `20` | 设置关系型数据库底层连接池最大容量。 |
| `init_tracing(bool)` | `bool` | `true` | 是否由框架自动初始化 `tracing_subscriber`。 |
| `register_subsystem(S)` | `SubsystemModule` | - | 注册业务领域子系统（自动合并模型元数据与路由）。 |
| `register_hook(H)` | `MutationHook` | - | 注册全局或领域突变生命周期钩子。 |
| `with_external_systems_dir(...)` | `PathBuf` | `[]` | 指定扫描外部子系统元数据清单的磁盘目录。 |
| `with_admin_spa_dir(...)` | `PathBuf` | `[]` | 覆写管理控制台前端 SPA 静态资源的磁盘服务路径。 |

---

## 4. 运行时动态业务专题配置 (Dynamic Configs)

在企业运营中，许多业务参数需要随时在线调整（例如：是否开启活动报名、每日抽奖次数上限、客服微信号、订单自动关闭超时时间等）。如果硬编码在环境变量中，每次调整都必须重启应用甚至重新构建容器，会导致业务短暂停机。

Foundry 提供了 **动态专题配置中心 (Dynamic Subsystem Configs)**：
- **Schema 驱动**：定义配置项的 Key、展示名称、数据类型（String, Number, Boolean, JSON）、默认值与校验规则；
- **热更新与聚合读取**：管理后台修改后实时入库，应用业务代码热读取，**零停机、即时生效**。

### 4.1 动态配置 Schema 定义

管理员可通过 API 或管理后台定义子系统的配置规格：

```http
POST /api/admin/s/{system_slug}/configs/schema
Authorization: Bearer <ADMIN_JWT_TOKEN>
Content-Type: application/json

{
  "key": "order_auto_close_minutes",
  "label": "未支付订单自动关闭时限 (分钟)",
  "field_type": "number",
  "default_value": "30",
  "description": "用户下单后未支付超过此时限将自动关闭并归还库存",
  "is_required": true
}
```

支持的 `field_type` 类型：
- `"string"`: 普通文本（如公告文案、客服联系方式）
- `"number"`: 数值（如时限、折扣率、限购数量）
- `"boolean"`: 开关（`true` / `false`，如活动开启状态）
- `"json"`: 复杂嵌套数据（如配送费规则表格、轮播图列表）

### 4.2 业务代码中读取动态配置

在领域服务或控制器中，直接通过注入的 `Database` 门面调用 `configs()` 即可获取当前最新的配置快照：

```rust
use foundry::prelude::*;

pub async fn check_order_timeout(ctx: &SystemContext, db: &Database) -> AppResult<i64> {
    // 1. 获取当前子系统的全部聚合配置映射 (Key -> Value)
    let configs = db.configs().get_aggregated(&ctx.system_slug).await?;

    // 2. 读取指定配置项并提供保底兜底值
    let timeout_minutes: i64 = configs
        .get("order_auto_close_minutes")
        .and_then(|v| v.as_i64())
        .unwrap_or(30);

    info!(%timeout_minutes, "当前订单超时配置生效中");
    Ok(timeout_minutes)
}
```

### 4.3 动态配置的 HTTP 聚合操作接口

客户端或第三方服务可直接调用标准的聚合配置接口：

#### 获取当前子系统全部生效配置
```http
GET /api/s/{system_slug}/configs
```
响应示例：
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "system_slug": "shop",
    "configs": {
      "order_auto_close_minutes": 30,
      "enable_flash_sale": true,
      "customer_service_wechat": "FoundryHelper"
    }
  }
}
```

#### 批量更新子系统配置值（需要管理员鉴权）
```http
PUT /api/s/{system_slug}/configs
Authorization: Bearer <ADMIN_JWT_TOKEN>
Content-Type: application/json

{
  "configs": {
    "order_auto_close_minutes": 45,
    "enable_flash_sale": false
  }
}
```

---

## 5. 生产环境安全与调优检查清单

在将应用部署到生产环境之前，请务必核对以下配置项：

1. **[必须] 更换生产 JWT 秘钥**：
   - 严禁在生产环境中使用默认的 `JWT_SECRET`。
   - 建议使用至少 32 字节以上的强随机字符串：
     ```bash
     openssl rand -base64 48
     ```
2. **[必须] 配置合理的最大连接池大小 (`db_pool_size`)**：
   - 默认连接池大小为 20。若生产服务采用多 Pod / 多实例部署，请确保所有实例的 `db_pool_size` 之和不超过数据库服务器的 `max_connections` 限制。
3. **[推荐] 关闭底层库的调试日志**：
   - 生产环境中将 `sqlx` 与 `tower_http` 设为 `warn` 或 `info`，避免高并发下大量 SQL 输出造成磁盘 I/O 阻塞：
     ```bash
     RUST_LOG="warn,my_app=info,foundry=info,foundry_engine=info"
     ```
4. **[按需] 配置 SMTP 邮件中继**：
   - 如果管理后台需要启用安全双因素验证（邮箱验证码登录、绑定安全邮箱），请配置稳定可靠的企业 SMTP（如 SendGrid、Amazon SES、阿里云邮件推送等）。

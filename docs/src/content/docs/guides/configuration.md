---
title: Application Configuration & Environment Variables Guide
description: "Complete reference for Foundry runtime configuration and environments: environment variables cheat sheet, FoundryBuilder fluent API, connection pool tuning, and runtime dynamic subsystem configs."
---

# Application Configuration & Environment Variables Guide

Foundry adheres to the philosophy of **"Convention Over Configuration, Minimalist yet Highly Controllable"**. Configuration is logically segmented into two tiers:
1. **Static Infrastructure Configuration**: Configured via system environment variables, `.env` files, and the fluent `FoundryBuilder` API to define network bindings, database connection strings, Redis, JWT secrets, and SMTP relays.
2. **Runtime Dynamic Subsystem Configuration**: Driven by `ConfigSchema` within the dynamic configuration center, allowing business administrators to view and adjust operational parameters in real time without restarting the server.

---

## 1. Environment Variables Complete Reference

During local development, Foundry automatically loads the project root `.env` file via `dotenvy`. In containerized environments like Docker or Kubernetes, it prioritizes operating system environment variables.

Here is the complete reference table of supported environment variables:

| Variable | Type | Default | Description & Examples |
|---|---|---|---|
| `HOST` | String | `0.0.0.0` | Network interface IP address to listen on. Use `127.0.0.1` for local development; keep `0.0.0.0` in containers. |
| `PORT` | u16 | `8080` | TCP port to bind the HTTP server to. |
| `DATABASE_URL` | String | `postgres://postgres:postgrespassword@localhost:5432/foundry` | Storage connection string. Automatically activates the appropriate SPI driver based on protocol prefix (`postgres://`, `mysql://`, `mariadb://`, `mongodb://`). |
| `REDIS_URL` | String | *(None / Optional)* | Redis cache connection string (e.g., `redis://127.0.0.1:6379`). If unset, the server gracefully runs in cacheless mode. |
| `JWT_SECRET` | String | `foundry_super_secret_jwt_key_2026_change_in_production` | Secret key for signing admin JWT tokens. **⚠️ Must be replaced with a strong random secret in production!** |
| `AUTO_MIGRATE` | bool | `true` | Whether to automatically run baseline schema migrations upon server startup (`true`, `1`, `false`, `0`). |
| `SMTP_HOST` | String | *(None / Optional)* | SMTP mail server hostname (e.g., `smtp.example.com`). Activates email verification code login and email binding. |
| `SMTP_PORT` | u16 | `587` | SMTP mail server port (common ports: `587` TLS, `465` SSL, `25` plain). |
| `SMTP_USER` | String | *(None / Optional)* | SMTP authentication username. |
| `SMTP_PASS` | String | *(None / Optional)* | SMTP authentication password or App Password. |
| `SMTP_FROM` | String | `Foundry <no-reply@foundry.local>` | Outgoing sender header display name and address. |
| `RUST_LOG` | String | `info,foundry=debug,foundry_engine=debug` | Dynamic log level filtering directives. See [Logging Guide](./logging/). |

---

## 2. `.env` Configuration Template

Create a `.env` file in your application root:

```bash
# ==============================================================================
# FOUNDRY PLATFORM CONFIGURATION
# ==============================================================================

# 1. Server Listener
HOST=127.0.0.1
PORT=8080

# 2. Database Connection URL (Choose your target engine)
# PostgreSQL:
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
# MySQL (8.0+):
# DATABASE_URL=mysql://root:root@localhost:3306/foundry
# MariaDB (10.5+):
# DATABASE_URL=mariadb://root:root@localhost:3307/foundry
# MongoDB (6.0+):
# DATABASE_URL=mongodb://127.0.0.1:27017/foundry

# 3. Auto-run migrations on startup
AUTO_MIGRATE=true

# 4. Optional Redis Cache
REDIS_URL=redis://127.0.0.1:6379

# 5. JWT Secret Key
JWT_SECRET=super_secret_jwt_key_for_my_production_app_2026

# 6. Optional SMTP Email Service
# SMTP_HOST=smtp.gmail.com
# SMTP_PORT=587
# SMTP_USER=notifications@my-domain.com
# SMTP_PASS=your_app_specific_password
# SMTP_FROM=Foundry Platform <notifications@my-domain.com>

# 7. Logging directives
RUST_LOG=info,my_app=debug,foundry=debug,foundry_engine=debug
```

---

## 3. Programmatic Configuration (`FoundryBuilder`)

In addition to environment variables, you can override settings programmatically using the fluent methods on `FoundryApp::builder()`:

```rust
use foundry::prelude::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load baseline config from environment variables
    let base_config = FoundryConfig::from_env();

    // 2. Assemble and override settings via fluent builder
    let app = FoundryApp::builder()
        .config(base_config)
        .host("0.0.0.0")
        .port(8080)
        .db_pool_size(50)
        .jwt_expiry_hours(72)
        .auto_migrate(true)
        .with_external_systems_dir(PathBuf::from("./external_systems"))
        .with_admin_spa_dir(PathBuf::from("./static/admin"))
        .register_subsystem(crate::systems::ShopSubsystem)
        .build()
        .await?;

    // 3. Start serving traffic
    app.run().await?;
    Ok(())
}
```

### 3.1 Builder Methods Reference

| Method | Argument Type | Default Behavior | Description |
|---|---|---|---|
| `config(FoundryConfig)` | `FoundryConfig` | `FoundryConfig::default()` | Replaces base configuration struct. |
| `host(impl Into<String>)` | `&str` / `String` | Reads `HOST` or `0.0.0.0` | Sets HTTP server host IP. |
| `port(u16)` | `u16` | Reads `PORT` or `8080` | Sets HTTP server listening port. |
| `database_url(...)` | `&str` / `String` | Reads `DATABASE_URL` | Sets target database URL. |
| `redis_url(Option<String>)` | `Option<String>` | Reads `REDIS_URL` | Sets Redis connection string. |
| `jwt_secret(...)` | `&str` / `String` | Reads `JWT_SECRET` | Sets secret key for JWT verification. |
| `jwt_expiry_hours(i64)` | `i64` | `168` (7 days) | Sets issued JWT token lifetime in hours. |
| `auto_migrate(bool)` | `bool` | `true` | Controls startup database migration execution. |
| `db_pool_size(u32)` | `u32` | `20` | Sets max connection pool capacity for relational engines. |
| `init_tracing(bool)` | `bool` | `true` | Whether the framework should initialize `tracing_subscriber`. |
| `register_subsystem(S)` | `SubsystemModule` | - | Registers a modular domain subsystem. |
| `register_hook(H)` | `MutationHook` | - | Registers lifecycle mutation hooks. |
| `with_external_systems_dir(...)` | `PathBuf` | `[]` | Scans filesystem directory for external manifest subsystems. |
| `with_admin_spa_dir(...)` | `PathBuf` | `[]` | Serves custom Admin UI static frontend SPA builds from disk. |

---

## 4. Runtime Dynamic Subsystem Configuration

In enterprise applications, operational parameters often need to be modified in real time (e.g., registration toggles, daily raffle limits, customer support IDs, or unpaid order timeout thresholds). If these parameters are stored in static environment variables, changing them requires rebuilding containers or restarting instances.

Foundry provides the **Dynamic Subsystem Configuration Center**:
- **Schema-Driven**: Define configuration keys, labels, types (string, number, boolean, json), and validation rules.
- **Hot-Reloading**: Changes committed in the Admin Console are persisted immediately and read dynamically by business services with **zero downtime**.

### 4.1 Schema Definition via API

```http
POST /api/v1/admin/s/{system_slug}/configs/schema
Authorization: Bearer <ADMIN_JWT_TOKEN>
Content-Type: application/json

{
  "key": "order_auto_close_minutes",
  "label": "Unpaid Order Timeout (Minutes)",
  "field_type": "number",
  "default_value": "30",
  "description": "Orders left unpaid beyond this threshold are automatically cancelled.",
  "is_required": true
}
```

### 4.2 Reading Dynamic Configs in Rust Logic

Access the latest configuration snapshot via `Database::configs()`:

```rust
use foundry::prelude::*;

pub async fn check_order_timeout(ctx: &SystemContext, db: &Database) -> AppResult<i64> {
    // 1. Fetch aggregated key-value config map for current subsystem
    let configs = db.configs().get_aggregated(&ctx.system_slug).await?;

    // 2. Read specific key with a fallback default value
    let timeout_minutes: i64 = configs
        .get("order_auto_close_minutes")
        .and_then(|v| v.as_i64())
        .unwrap_or(30);

    info!(%timeout_minutes, "Active order timeout loaded");
    Ok(timeout_minutes)
}
```

### 4.3 Aggregated HTTP Endpoints

#### Read Active Configurations
```http
GET /api/v1/s/{system_slug}/configs
```

#### Batch Update Configurations (Admin Protected)
```http
PUT /api/v1/s/{system_slug}/configs
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

## 5. Production Checklist

1. **[Critical] Rotate JWT Secret**:
   Generate a cryptographically secure random string with at least 32 bytes:
   ```bash
   openssl rand -base64 48
   ```
2. **[Critical] Tune Connection Pool Size (`db_pool_size`)**:
   Ensure `instances * db_pool_size < database_max_connections`.
3. **[Recommended] Lower Driver Logging Verbosity**:
   Suppress database and transport debug logs in production:
   ```bash
   RUST_LOG="warn,my_app=info,foundry=info,foundry_engine=info"
   ```
4. **[Optional] Configure SMTP Relay**:
   Enable 2FA login codes and email binding for administrative accounts.

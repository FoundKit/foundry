# Foundry Architecture Blueprint

> **Organization**: [foundkit](https://github.com/foundkit)  
> **Project**: `foundry`  
> **Tagline**: *Build complete systems from a common foundation.*  
> **Repository Description**: *Foundry is an open-source platform for building and running multiple independent backend systems from a shared foundation.*

---

## 1. Product Vision & Positioning

### 1.1 What is Foundry?
**Foundry** is a complete, self-contained, open-source **Multi-System Backend Platform & Management Product** (Backend-as-a-Service / Multi-Tenant Infrastructure Engine).

Unlike single-project BaaS solutions (such as Strapi, Directus, PocketBase, or Supabase) which deploy one instance per content model scope, **Foundry is built from the ground up as a Monorepo product to manage multiple independent systems from a unified foundation.**

```
+-------------------------------------------------------------------------------------------------------+
|                                        FOUNDRY PLATFORM ECOSYSTEM                                     |
|                                                                                                       |
|  +--------------------------------+  +-------------------------------------------------------------+  |
|  |     Foundry Admin UI (SPA)     |  |         Client Layer (REST-First & OpenAPI-Native)          |  |
|  |  - Visual System Builder       |  |  - Standard HTTP Clients (Fetch, Axios, Ktor, cURL, etc.)   |  |
|  |  - Zero-DDL Dynamic Models     |  |  - Type-Safe Generated Clients via OpenAPI 3.0 Specs        |  |
|  |  - Admin & Topic RBAC Manager  |  |  - Sub-System Custom Domain APIs                            |  |
|  |  - Non-GET Audit Log Viewer    |  |                                                             |  |
|  +---------------+----------------+  +------------------------------+------------------------------+  |
|                  |                                                  |                                 |
|                  +------------------------+-------------------------+                                 |
|                                           | Standard RESTful / OpenAPI (Utoipa)                       |
|                                           v                                                           |
|  +-------------------------------------------------------------------------------------------------+  |
|  |                              Foundry Server Engine (Rust Monorepo)                              |  |
|  |                                                                                                 |  |
|  |  +--------------------+  +--------------------+  +--------------------+  +-------------------+  |  |
|  |  | Sub-System Alpha   |  | Sub-System Beta    |  | Sub-System Gamma   |  | Custom Backend N  |  |  |
|  |  | (Configs, Models,  |  | (Configs, Models,  |  | (Custom Logic, DTO,|  | (Self-Contained   |  |  |
|  |  |  Auto-CRUD APIs)   |  |  Auto-CRUD APIs)   |  |  Domain Models)    |  |  Business Logic)  |  |  |
|  |  +--------------------+  +--------------------+  +--------------------+  +-------------------+  |  |
|  |                                                                                                 |  |
|  |  +-------------------------------------------------------------------------------------------+  |  |
|  |  |                            Core Platform Infrastructure Services                          |  |  |
|  |  |  - Multi-Tenant Router (`/api/v1/s/:slug`)   - Zero-DDL Storage Engine (Postgres JSONB/GIN)|  |  |
|  |  |  - Admin IAM & Topic-Scoped RBAC             - Non-GET Write Operation Audit Interceptor   |  |  |
|  |  |  - Wasmtime Dynamic Sandbox & Trait Hooks    - Compile-Time OpenAPI 3.0 Aggregator         |  |  |
|  |  +-------------------------------------------------------------------------------------------+  |  |
|  +-------------------------------------------------------------------------------------------------+  |
+-------------------------------------------------------------------------------------------------------+
```

### 1.2 Core Value Propositions
1. **Multi-Tenant System Engine ("一平台管多系统")**:
   Run and manage $N$ independent sub-systems (App backends, mini-programs, marketing campaign sites) within a single deployed Foundry instance with strict tenant isolation.
2. **Visual One-Click System & Model Generation (后台建系统 + 自动 CRUD)**:
   A modern Web Admin Panel allowing users to graphically construct sub-systems, define dynamic data schemas, and automatically expose high-performance RESTful APIs with fine-grained access control toggles.
3. **Code-First Custom Endpoint & Logic Extensibility ("专题自定义代码与业务扩展")**:
   First-class support for creating dedicated sub-system code packages with distinct unique identifiers (`system_slug`), custom controller directories (`controllers/`), business logic services (`logic/`), and DTO validation layers (`dto/`), seamlessly integrated with shared infrastructure and Axum routing.
4. **Pure Infrastructure Foundation & Sub-System Domain Autonomy ("纯净基建底座与子系统业务自治")**:
   Foundry is strictly positioned as a generic, unopinionated infrastructure platform. In the foundational phase, the platform does not bundle redundant public business modules (no platform-wide points tables, no form tables, and no unified end-user authentication center). Each sub-system possesses complete autonomy to design its own user authentication, domain entities, and business logic.
5. **Hierarchical Admin IAM & Topic-Scoped RBAC ("分级管理员与专题授权")**:
   Built-in administrator authorization model. System initialization provisions a Super Admin (`super_admin`) with omnipotent privileges across all sub-systems (`["*"]`). The Super Admin can create normal administrators (`admin`) and delegate management rights to specific topics (`allowed_systems: ["carnival_2026"]`).
6. **Comprehensive Non-GET Operation Audit Trail ("全量管理员写操作与状态审计")**:
   Security-first auditing middleware that automatically captures every non-GET request (POST, PUT, PATCH, DELETE, login) executed by administrators, recording the operator, target sub-system, path, dynamic operation name, full request payload/parameters, IP, status code, and latency.
7. **Hybrid Extension Engine (Rust Traits + Wasmtime Sandbox)**:
   Extend any sub-system using native **Rust traits** for maximum performance or sandboxed **Wasm plugins** for runtime hot-reloading without server restarts.
8. **REST-First & OpenAPI-Native Architecture**:
   Zero client-vendor lock-in. Clean, standardized REST endpoints with auto-generated OpenAPI 3.0 definitions allowing clients to connect directly without mandatory SDK dependencies.

---

## 2. Monorepo Repository Structure

Foundry is organized as a unified Monorepo containing the Rust Backend Engine, Sub-System Custom Code Packages, Visual Web Admin Dashboard SPA, Developer CLI, and Modular Backend Crates.

```
foundry/                         # Monorepo Root
├── README.md                    # Product overview & getting started guide
├── docs/                        # Architecture & development documentation
│   ├── architecture.md          # Full product architecture blueprint
│   └── todo.md                  # Development roadmap & task tracker
├── migrations/                  # Database initialization & migrations
│   └── init.sql                 # Baseline PostgreSQL schema (Zero-DDL tables)
├── apps/                        # Executable Applications
│   ├── server/                  # Foundry Core Server (Rust / Axum binary entry point)
│   ├── admin/                   # Visual Web Admin Dashboard SPA (React / Vite / Tailwind)
│   │   ├── src/
│   │   │   ├── components/      # UI component library & design system (Shadcn UI / Tailwind)
│   │   │   ├── locales/         # i18n dictionaries (en-US, zh-CN, community packs)
│   │   │   ├── pages/           # Admin views (Schema Builder, Data Explorer, Admin IAM, Audit Logs)
│   │   │   └── services/        # API client bindings & state management
│   │   ├── package.json         # Frontend dependencies & scripts
│   │   └── vite.config.ts       # Vite build configuration
│   └── cli/                     # Developer CLI Tool (System scaffolding, code generation, migration)
├── systems/                     # Sub-System Custom Code Workspace (专题自定义代码模块目录)
│   ├── Cargo.toml               # Systems Workspace / Crate Definition
│   ├── src/
│   │   ├── lib.rs               # Sub-system registry & static router auto-mount loader
│   │   └── <system_slug>/       # Dedicated sub-system directory (e.g., carnival_2026, vip_mall)
│   │       ├── mod.rs           # Subsystem registration & SubsystemModule trait implementation
│   │       ├── controllers/     # Custom HTTP Controllers (Axum Handlers + Utoipa OpenAPI annotations)
│   │       │   ├── mod.rs       # Controller route registration & router export
│   │       │   └── custom_controller.rs
│   │       ├── logic/           # Custom Business Logic & Service Layer (Domain rules & transactions)
│   │       │   ├── mod.rs
│   │       │   └── custom_service.rs
│   │       └── dto/             # Request/Response DTOs & Validation schemas
│   │           ├── mod.rs
│   │           └── custom_dto.rs
├── crates/                      # Modular Backend Core Crates (Rust Workspace)
│   ├── foundry_core/            # SystemContext, SubsystemModule trait, primitives & error definitions
│   ├── foundry_storage/         # Dynamic schema engine, ORM abstraction (SeaORM / SQLx)
│   ├── foundry_auth/            # Admin IAM, JWT/Argon2id, Super Admin & Topic-Scoped RBAC
│   ├── foundry_engine/          # Multi-system router, Auto-CRUD API generator, Non-GET Audit Middleware
│   └── foundry_extension/       # Wasmtime runtime sandbox & Rust hook pipeline
└── docker/                      # Containerization & Deployment
    ├── Dockerfile               # Production multi-stage build (Server + Admin UI bundle)
    └── docker-compose.yml       # Local dev setup (Foundry + PostgreSQL 18.6+ + Redis 8+)
```

---

## 3. Key Architecture & System Design

### 3.1 Multi-Tenant Sub-System Architecture & Unique Identifiers
Every project or application managed inside Foundry is defined as a **Sub-System** (专题 / 子系统).

#### 3.1.1 Sub-System Unique Identifier Standards (`system_slug` & `system_id`)
To ensure reliable multi-tenant isolation, automated route mounting, code organization, and cache scoping, each Sub-System is identified by two canonical keys:

1. **`system_id` (Internal Immutable ID)**:
   - Type: `UUIDv7` or `UUID` (e.g., `018f3a8b-7c9d-7a2e-b14e-6e82c5d12345`).
   - Purpose: Primary key in the database metadata table (`systems`), immutable database foreign keys, and internal tenant scoping.
2. **`system_slug` (Human-Readable Code & URL Identifier)**:
   - Format & Validation: Lowercase alphanumeric string with underscores or hyphens (`^[a-z0-9_-]{2,32}$`, e.g., `carnival_2026`, `vip_mall`, `alpha`).
   - Global Uniqueness: Unique across the entire platform instance; once created, it cannot be modified to avoid breaking code references and URLs.
   - **Unified Scoping Rules Across Layers**:
     - **Code Directory Binding**: Dedicated code module in `systems/src/<system_slug>/`.
     - **RESTful API Routing Prefix**: Directly mapped to `/api/v1/s/{system_slug}/...`.
     - **PostgreSQL Isolation**: Dynamic storage scoped by `system_id` (or `system_slug`) in `system_configs` and `model_records`.
     - **Redis Cache Namespace**: Strict key prefixing `foundry:{system_slug}:*` (e.g., `foundry:carnival_2026:session:1001`).
     - **OpenAPI / Swagger Grouping**: Automatically aggregated under OpenAPI tag `System: {system_slug}`.
     - **Observability / Tracing**: Structured logging, audit trails, and OpenTelemetry traces automatically attach `system_slug`.

```
                        [ Incoming Request ]
                                 │
                     ┌───────────┴───────────┐
                     ▼                       ▼
           Path: /api/v1/s/:system_slug/...  Header: X-Foundry-System-ID
                     │                       │
                     └───────────┬───────────┘
                                 ▼
                     [ Axum SystemContext ]
                                 │
     ┌───────────────────────────┼───────────────────────────┐
     ▼                           ▼                           ▼
[ Auto-CRUD Routes ]   [ Custom System Handlers ]   [ Topic RBAC / Guard ]
(Dynamic Schema DB)    (systems/<system_slug>/...)    (allowed_systems check)
     │                           │                           │
     └───────────────────────────┼───────────────────────────┘
                                 ▼
                    [ Storage / Query Execution ]
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       PostgreSQL Records / Configs       System Redis Namespace
       (`model_records`/`system_configs`) (e.g., `foundry:alpha:*`)
```

---

### 3.2 Dynamic Schema Engine & Zero-DDL Storage Architecture

Database capability is a native, zero-boilerplate foundational engine in Foundry. Developers and administrators **never need to write repetitive CRUD controllers or manual SQL queries**.

#### 3.2.1 Enterprise Production Reality: Zero-DDL Architecture
In enterprise production environments, applications operate under strict DBA governance:
- **No Runtime DDL Permissions**: Application database users are strictly restricted to DML operations (`SELECT`, `INSERT`, `UPDATE`, `DELETE`). DDL (`CREATE TABLE`, `ALTER TABLE`) is prohibited to eliminate metadata locks, connection exhaustion, and accidental schema destruction.
- **One-Time Initialization**: All platform tables are deployed once via standard pre-audited DDL script ([`migrations/init.sql`](file:///home/panhy/src/foundkit/foundry/migrations/init.sql)).
- **Zero-DDL Runtime Agility**: Administrators and developers can create unlimited sub-systems, models, and custom fields in the Admin UI without running any database DDL or requiring DBA intervention.

```
+───────────────────────────────────────────────────────────────────────────────────────────+
|                           FOUNDRY ZERO-DDL STORAGE RUNTIME                                |
|                                                                                           |
|  [ 1. System Configs (专题专属配置项) ]                                                    |
|    └── `system_configs`: Key-Value config properties (banner, rules, times, toggles)      |
|                                                                                           |
|  [ 2. Business Data Models (海量业务数据模型) ]                                            |
|    ├── `models` & `model_fields`: Schema metadata & form widget specifications            |
|    └── `model_records`: Multi-tenant JSONB record storage + GIN Inverted Indexing         |
|                                                                                           |
|  [ Access Layer (Auto-CRUD & Programmatic ORM) ]                                          |
|    ├── Configs REST API: `GET /api/v1/s/{slug}/configs` & `PUT /api/v1/s/{slug}/configs`   |
|    ├── Auto-CRUD REST API: `/api/v1/s/{slug}/{model_slug}` (List, Detail, Create, Update) |
|    └── Rust Logic: `ctx.configs()` & `ctx.model("{model_slug}")`                          |
+───────────────────────────────────────────────────────────────────────────────────────────+
```

---

#### 3.2.2 Decoupled Architecture: System Configs vs. Business Data Models

Foundry completely separates lightweight topic configurations from multi-record business data entities:

```
                                  [ Sub-System Storage ]
                                             │
                   ┌─────────────────────────┴─────────────────────────┐
                   ▼                                                   ▼
     [ 1. 专题专属配置 (System Configs) ]                [ 2. 业务数据模型 (Data Models) ]
     - 归属于系统自身的全局轻量配置项                     - 独立的业务数据实体 (商品、文章、日志、自定义数据)
     - 整个专题仅有几个到十几个配置项                     - 存储成千上万条结构化数据行
     - 物理底层：`system_configs` 表                     - 物理底层：`models` + `model_fields` + `model_records`
     - API: 聚合为单一 JSON 对象                         - API: 标准 Auto-CRUD (分页/过滤/排序)
       `GET/PUT /api/v1/s/{slug}/configs`                 `GET/POST/PUT/DELETE /api/v1/s/{slug}/{model}`
```

##### 1. System Configs (专题专属配置)
- **Use Case**: Global parameters for a campaign/sub-system (e.g. Campaign Title, Banner Image URL, Event Time Range, Share Text, RichText Rules, Switch Flags).
- **Physical Storage**: Stored in `system_configs` (`system_id`, `key`, `label`, `value_type`, `value: JSONB`, `options`, `sort_order`).
- **Clean Aggregation**: The backend aggregates all configs into a single key-value JSON response on `GET /api/v1/s/{slug}/configs`.
- **Visual Form UI (严禁裸 JSON 编辑器)**: The Admin UI renders dedicated visual widgets according to `value_type` and `options` (Image Uploader, DatePicker, Switch, RichText, Array Repeater).

##### 2. Business Data Models (业务数据模型)
- **Use Case**: Entities requiring multiple structured records (e.g., Product Catalog, Article List, Custom Submissions, Operation Logs).
- **Physical Storage**:
  - `models`: Model registry & permissions.
  - `model_fields`: Field specifications, data types, and form widget configuration.
  - `model_records`: Record rows stored with `data: JSONB` and PostgreSQL GIN inverted indexing.
- **Query Capabilities**: Full RESTful Auto-CRUD with pagination, multi-field filtering (`filter[field][op]=val`), multi-column sorting, keyword search, and GIN index scans.

---

#### 3.2.3 Comprehensive Field & Config Type Matrix

Both System Configs and Data Models support a rich set of first-class field types:

| Field Type | Storage Representation | Visual Form Widget in Admin UI | Validation & Features |
| :--- | :--- | :--- | :--- |
| **`string`** (单行文本) | JSON string (`"title"`) | Standard `<Input />` box | Min/max length, regex pattern, trim |
| **`richtext`** (富文本/HTML)| JSON string (`"<p>...</p>"`) | WYSIWYG Editor (TipTap / Quill) | Sanitization, embedded image upload |
| **`image`** (单图片) | CDN URL string (`"https://..."`) | Image Uploader with preview & crop | Direct Qiniu/OSS/S3/Local upload |
| **`file`** (媒体附件) | Asset URL / Object | File Dropzone & Progress bar | MIME type filter, size limit |
| **`integer`** (整型) | JSON integer (`42`) | Stepper `<InputNumber />` | Range constraints (`min`, `max`), step |
| **`number`** / **`float`** | JSON number (`99.50`) | Decimal Precision Input | Float precision, currency format |
| **`boolean`** (布尔开关) | JSON boolean (`true`/`false`)| Toggle `<Switch />` | Default state, active/inactive labels |
| **`datetime`** / **`date`** | ISO-8601 string / Timestamp | Calendar & Time Picker | Timezone formatting, future/past rules |
| **`array`** (动态数组/列表) | JSON array (`[...]`) | **Dynamic Visual Repeater List** | Array of strings, images, or sub-objects with `+ Add Item` & drag-and-drop sorting |
| **`relation`** (关联关系) | Foreign ID / Array of IDs | Searchable Select / Modal picker | 1:1, 1:N relations to Data models |

> [!IMPORTANT]
> **No Raw JSON UX Rule**: Operators and content managers NEVER edit raw JSON strings. Every field—even complex arrays or image galleries—is managed via purpose-built visual components. The system handles the serialization to/from JSON transparently.

---

#### 3.2.4 Dual-Mode Access API & Programmatic Rust Abstractions

##### 1. RESTful HTTP Endpoints
- **Data Models (Multi-Record Collection)**:
  - `GET /api/v1/s/{system_slug}/{model_slug}` (Paginated list with filter & sort)
  - `POST /api/v1/s/{system_slug}/{model_slug}` (Create record with schema validation)
  - `GET /api/v1/s/{system_slug}/{model_slug}/:id` (Get record by ID)
  - `PUT/PATCH /api/v1/s/{system_slug}/{model_slug}/:id` (Update record)
  - `DELETE /api/v1/s/{system_slug}/{model_slug}/:id` (Soft/hard delete)
- **System Configs (Aggregated Settings)**:
  - `GET /api/v1/s/{system_slug}/configs` (Get aggregated configuration JSON)
  - `PUT /api/v1/s/{system_slug}/configs` (Update multiple configuration values with validation)

##### 2. Programmatic Access in Custom Rust Logic (`systems/src/<slug>/logic/`)
```rust
// 1. Working with Data Models
let product_model = ctx.model("products")?;
let product = product_model.find_one()
    .where_eq("id", product_id)
    .where_gt("stock", 0)
    .execute(&ctx.db)
    .await?;

product_model.update_by_id(product_id)
    .decrement("stock", 1)
    .execute(&ctx.db)
    .await?;

// 2. Working with System Configs
#[derive(Deserialize, Serialize)]
pub struct SubsystemSettings {
    pub banner_image: String,
    pub is_active: bool,
    pub support_email: String,
}

// Strongly-typed retrieval into Rust structs
let config: SubsystemSettings = ctx.configs().get().await?;

// Or raw JSON value retrieval
let banner = ctx.configs().get_string("banner_image").await?;
```

---

### 3.3 Admin IAM, Topic-Scoped RBAC & Non-GET Audit Logging (管理员权限与审计系统)

Foundry implements an enterprise-grade administrative identity and audit system focused on security, tenant boundary enforcement, and complete state mutation tracking.

```
+───────────────────────────────────────────────────────────────────────────────────────────+
|                         ADMIN IAM & NON-GET AUDIT SYSTEM ARCHITECTURE                     |
|                                                                                           |
|  [ Admin IAM & Topic-Scoped RBAC (`admins`) ]                                             |
|    ├── Super Admin (`role: super_admin`, `allowed_systems: ["*"]`)                        |
|    │   └── Full platform privileges: Create/manage sub-systems, add admins, grant scopes  |
|    └── Normal Admin (`role: admin`, `allowed_systems: ["carnival_2026", "vip_mall"]`)    |
|        └── Scoped privileges: Manage configs, models, & records ONLY in assigned topics   |
|                                                                                           |
|  [ Non-GET Audit Interceptor Middleware (`audit_logs`) ]                                  |
|    ├── Intercepts: All non-GET requests (POST, PUT, PATCH, DELETE, login)                |
|    ├── Discrete Storage: headers, query_params, body_params stored in separate columns   |
|    ├── Captures: admin_id, username, system_slug, method, path, action_name,             |
|    │             headers, query_params, body_params, ip_address, ua, status, duration    |
|    └── Dynamic Action Mapping: Flexible code-level route-to-action dictionary             |
|        (e.g., `/admin/auth/login` -> "登录", `/admin/s/:slug/configs` -> "修改专题配置")  |
+───────────────────────────────────────────────────────────────────────────────────────────+
```

#### 3.3.1 Admin IAM & Hierarchical Topic Authorization

1. **Unified Admin Identity (`admins` Table)**:
   - All management operations in Foundry are executed by administrators registered in the `admins` table.
   - Passwords are encrypted using high-security **Argon2id**.
   - Sessions are authenticated via signed JWT bearer tokens with embedded `admin_id`, `role`, and `allowed_systems` claims.

2. **Hierarchical Roles & Permissions**:
   - **Super Admin (`super_admin`)**:
     - System initialization provisions a default super admin (`admin` / `admin123456`).
     - Possesses global wildcard permissions (`allowed_systems: ["*"]`).
     - Can create sub-systems, create and manage other administrators, and assign topic scopes.
   - **Normal Admin (`admin`)**:
     - Created by the Super Admin.
     - Has an explicit whitelist of accessible sub-systems stored in `allowed_systems` (e.g. `["carnival_2026", "activity_2026_q1"]`).
     - Attempting to access or modify any sub-system outside `allowed_systems` immediately triggers a `403 Forbidden` error.

3. **RBAC Middleware Evaluation Flow**:
   ```rust
   // Pseudo-code for Axum Topic RBAC Middleware
   pub async fn require_system_access(
       State(ctx): State<SystemContext>,
       Extension(current_admin): Extension<AdminClaims>,
       req: Request,
       next: Next,
   ) -> Result<Response, StatusCode> {
       if current_admin.role == "super_admin" {
           return Ok(next.run(req).await);
       }
       
       if current_admin.allowed_systems.contains(&ctx.system_slug) {
           return Ok(next.run(req).await);
       }
       
       Err(StatusCode::FORBIDDEN)
   }
   ```

---

#### 3.3.2 Non-GET Operation Audit Logging (`audit_logs`)

To guarantee strict compliance, security forensics, and operational traceability, Foundry implements a dedicated **Non-GET Audit Logging Middleware**:

1. **Selective Interception Strategy**:
   - **GET Requests**: Excluded from audit storage to avoid database bloating from read traffic.
   - **Non-GET Requests**: Every `POST`, `PUT`, `PATCH`, `DELETE` request (and authentication endpoints like `/admin/auth/login`) is captured and persisted.

2. **Discrete & Raw Parameter Storage Model (`audit_logs`)**:
   Standard HTTP requests contain three distinct parameter dimensions. To accommodate non-JSON bodies (form-urlencoded, raw text, XML, etc.) and preserve original URL query strings without lossy transformations, query parameters and request bodies are stored in their **raw original text format**:

   ```
   [ Incoming Admin Non-GET Request ]
   e.g. POST /admin/s/carnival_2026/users?page=1&limit=10
        -H 'Content-Type: application/json' -H 'X-Request-ID: req_123'
        -d '{"name": "张三", "age": 30, "email": "zhangsan@example.com"}'
                    │
                    ▼
   [ audit_logs Table Record ]
   ├── `headers`      : {"Content-Type": "application/json", "X-Request-ID": "req_123", ...} (JSONB)
   ├── `query_params` : "page=1&limit=10" (VARCHAR(2048))
   └── `body_params`  : "{\"name\": \"张三\", \"age\": 30, \"email\": \"zhangsan@example.com\"}" (Raw TEXT)
   ```

   - `admin_id`: UUID of the acting administrator (nullable for login attempts prior to authentication).
   - `admin_username`: Username snapshot (`VARCHAR(48)`) for rapid rendering without repetitive joins.
   - `system_slug`: Target sub-system slug (`VARCHAR(32)`, NULL for global actions like admin management).
   - `method`: HTTP method (`VARCHAR(10)`, e.g., `POST`, `PUT`, `DELETE`).
   - `path`: Exact endpoint path (`VARCHAR(255)`, e.g., `/admin/s/carnival_2026/configs`).
   - `action_name`: Human-readable action description (`VARCHAR(64)`) resolved dynamically in application code.
   - `headers`: JSONB key-value map of request headers (e.g., `Content-Type`, `X-Request-ID`, with token sanitization).
   - `query_params`: Raw query string as sent by the client (`VARCHAR(2048)`), providing ample buffer while preventing memory abuse.
   - `body_params`: Raw request body content stored as `TEXT` without mandatory JSON parsing, safely preserving JSON, form-data, plain text, XML, and other formats.
   - `ip_address` & `user_agent`: Client network and browser metadata (`VARCHAR(45)` for IPv6, `VARCHAR(512)` for User-Agent).
   - `status_code` & `duration_ms`: Execution result code (`SMALLINT`) and latency in milliseconds (`INTEGER`).

3. **Dynamic Action Name Mapping in Application Code**:
   Rather than hardcoding rigid database ENUMs, the application uses a flexible route-matching registry that maps URL patterns and methods to human-readable names:
   ```rust
   pub fn resolve_action_name(method: &Method, path: &str) -> &'static str {
       match (method.as_str(), path) {
           ("POST", p) if p.ends_with("/auth/login") => "管理员登录",
           ("POST", p) if p.ends_with("/auth/logout") => "管理员登出",
           ("POST", "/admin/admins") => "新增管理员",
           ("PUT", p) if p.starts_with("/admin/admins/") => "修改管理员信息",
           ("POST", "/admin/systems") => "创建子系统",
           ("PUT", p) if p.contains("/configs") => "修改专题配置",
           ("POST", p) if p.contains("/models") => "创建数据模型",
           ("DELETE", p) if p.contains("/records/") => "删除数据记录",
           _ => "业务写操作",
       }
   }
   ```

---

#### 3.3.3 Sub-System Autonomy & Client Authentication Model

A core architectural principle of Foundry in the infrastructure phase is **Sub-System Domain Autonomy**:
- **No Pre-Packaged End-User Tables**: Foundry does NOT prescribe or enforce a platform-wide end-user database (`system_users` or unified login).
- **Sub-System Specific Needs**: Different sub-systems have vastly different user requirements (WeChat OpenID login, SMS OTP, OAuth2, anonymous guest sessions, or pure internal tools).
- **Self-Contained Implementation**: When a sub-system requires client user login or domain entities (e.g. membership profiles, order ledgers), it designs and maintains those models directly within its own sub-system workspace (`systems/src/<slug>/`) or dynamic models.

---

### 3.4 Sub-System Code-First Custom Extensibility Architecture (专题自定义代码与接口扩展架构)

In addition to dynamic Low-Code models (Auto-CRUD), Foundry provides first-class support for **Code-First Sub-System Extension**. Developers can write custom controllers and domain business logic in native Rust under a dedicated sub-system directory.

#### 3.4.1 Three-Layer Architectural Pattern per Sub-System
Each sub-system located in `systems/src/<system_slug>/` strictly adheres to a standard Three-Layer Architecture:

```
systems/src/<system_slug>/
├── mod.rs                  # [Entry & Registration] Implements SubsystemModule trait
├── controllers/            # [Layer 1: HTTP Presentation / API Controller]
│   ├── mod.rs              # Route assembly & sub-router export
│   └── custom_controller.rs # Axum Handlers, parameter extraction, OpenAPI annotations
├── logic/                  # [Layer 2: Domain Business Logic & Services]
│   ├── mod.rs              # Service exports & domain rules
│   └── custom_service.rs   # Core business rules, database transactions & custom workflows
└── dto/                    # [Layer 3: Data Transfer Objects & Validation]
    ├── mod.rs              # Request/Response structs, validator schemas
    └── custom_dto.rs       # Strongly typed payloads, serde deserialization, custom validation rules
```

1. **Controller Layer (`controllers/`)**:
   - **Responsibility**: HTTP protocol handling, URL path/query/JSON body parsing, parameter validation (`validator`), invoking the Logic Layer, and returning standard JSON response envelopes.
   - **OpenAPI Integration**: Annotated with `#[utoipa::path(...)]` for compile-time generation of OpenAPI documentation.
   - **Rule**: Controllers MUST NOT contain business logic or direct raw SQL queries; they delegate all orchestration to the Logic Layer.

2. **Logic / Service Layer (`logic/`)**:
   - **Responsibility**: Pure business domain logic, transactional state transitions, and custom domain orchestration.
   - **Access to Storage**: Leverages dynamic schema ORM (`ctx.model(...)`, `ctx.configs()`) or direct SQL connections scoped to the current `SystemContext`.
   - **Rule**: Framework-agnostic (independent of Axum HTTP types), easily unit-testable.

3. **DTO Layer (`dto/`)**:
   - **Responsibility**: Defines request inputs and response payloads with serde serialization/deserialization, default values, and declarative validation constraints (`#[validate(range(min = 1, max = 100))]`).

#### 3.4.2 `SubsystemModule` Trait & Custom Route Auto-Mounting
Foundry defines a standard Rust trait in `crates/foundry_core` that every sub-system implements:

```rust
// In crates/foundry_core/src/subsystem.rs
pub trait SubsystemModule: Send + Sync + 'static {
    /// Returns the unique slug of the sub-system (e.g. "carnival_2026")
    fn slug(&self) -> &'static str;
    
    /// Human-readable display name of the sub-system
    fn display_name(&self) -> &'static str;
    
    /// Mounts custom HTTP routes under the sub-system's scope
    /// Routes mounted here are accessible at `/api/v1/s/{slug}/...`
    fn register_routes(&self, router: axum::Router<SystemContext>) -> axum::Router<SystemContext>;
    
    /// Registers sub-system OpenAPI schemas & paths into the global Utoipa registry
    fn register_openapi(&self, openapi: &mut utoipa::openapi::OpenApi);
}
```

#### 3.4.3 Unified Dependency Injection & Context Extraction
Custom controllers receive the `SystemContext` automatically injected by Axum middleware:

```rust
// Example Controller Handler in systems/src/carnival_2026/controllers/custom_controller.rs
#[utoipa::path(
    post,
    path = "/api/v1/s/carnival_2026/participate",
    request_body = ParticipateRequest,
    responses(
        (status = 200, description = "Participation successful", body = ParticipateResponse),
        (status = 400, description = "Invalid request payload")
    ),
    tag = "System: carnival_2026"
)]
pub async fn handle_participate(
    State(ctx): State<SystemContext>,
    Json(payload): Json<ParticipateRequest>,
) -> Result<Json<ApiResponse<ParticipateResponse>>, AppError> {
    // 1. Validate incoming payload
    payload.validate()?;
    
    // 2. Delegate to Business Logic Layer
    let result = CarnivalService::participate(&ctx, payload).await?;
    
    // 3. Return standardized API response
    Ok(Json(ApiResponse::success(result)))
}
```

#### 3.4.4 Seamless Route Composition: Dynamic Auto-CRUD + Custom APIs
When a request arrives at `/api/v1/s/{system_slug}/*`, the `foundry_engine` router matches:
1. **Custom Sub-System Routes**: If a custom controller is registered in `systems/src/{system_slug}/controllers/` for that path, it handles the request with highest priority.
2. **Dynamic Auto-CRUD Routes**: If no custom endpoint matches, the engine checks dynamic schema models defined in the visual Admin UI (e.g. `/api/v1/s/{system_slug}/products`).
3. **404 Not Found Envelope**: If neither matches, returns a standard localized 404 error envelope.

#### 3.4.5 CLI Scaffolding Workflow
The Developer CLI (`apps/cli`) provides automated scaffolding for new sub-systems and endpoints:
```bash
# 1. Create a brand new sub-system code module with unique slug
foundry-cli system new <system_slug> --name "Marketing Carnival 2026"

# 2. Add a new controller to an existing sub-system
foundry-cli system add controller <system_slug> <controller_name>

# 3. Add a new logic service to an existing sub-system
foundry-cli system add logic <system_slug> <service_name>
```

---

### 3.5 Hybrid Extension Architecture (Rust Hooks + Wasmtime Sandbox)

Foundry enables runtime and non-compiled customization through two complementary extension mechanisms:

```
                          [ Model Mutation Request ]
                                       │
                                       ▼
                         [ Before Mutation Hook Pipeline ]
                                       │
                  ┌────────────────────┴────────────────────┐
                  ▼                                         ▼
      [ Native Rust Trait Plugin ]               [ Sandboxed WASM Plugin ]
      - In-process execution                     - Out-of-process memory sandbox
      - Zero overhead performance                - Hot-reloadable at runtime
      - Low-level system access                  - Safe user/tenant customization
                  │                                         │
                  └────────────────────┬────────────────────┘
                                       ▼
                            [ Database Operation ]
                                       │
                                       ▼
                         [ After Mutation Hook Pipeline ]
                                       │
                       (Audit Logs, Events, Webhooks)
```

1. **Native Rust Mutation Hooks**: In-process hooks (`before_create`, `after_update`, etc.) executing around database mutations for sub-systems.
2. **Wasmtime Sandboxed Plugins**: Allows runtime dynamic execution of sandboxed `.wasm` modules (written in Rust, Go, AssemblyScript, or C) with zero engine recompilation.

---

### 3.6 End-to-End Localization (i18n) Architecture

Foundry supports complete internationalization across all layers:

```
[ Client App / Browser ] ───(Accept-Language: zh-CN)───> [ Axum Middleware ]
                                                                │
                                    ┌───────────────────────────┴───────────────────────────┐
                                    ▼                                                       ▼
                        [ Error / Status Response ]                                [ Admin SPA UI ]
                        {                                                          - `apps/admin/src/locales`
                          "code": 40001,                                           - `react-i18next`
                          "message": "Resource not found",                         - Hot-swappable
                          "i18n_key": "errors.resource_not_found",                   language packs
                          "args": { "resource": "products" }
                        }
```

- **Frontend i18n (`apps/admin/src/locales` + `react-i18next`)**: Embedded modular translation files (`en-US.json`, `zh-CN.json`, `ja-JP.json`), dynamic locale bundle loader, and translation verification tools.
- **Backend Error Envelopes**: Errors return standardized JSON envelopes containing machine-readable error codes and localization keys:
  ```json
  {
    "code": 40001,
    "message": "Resource not found",
    "i18n_key": "errors.resource_not_found",
    "args": { "resource": "products" }
  }
  ```
- **Backend Formatting (`fluent-rs`)**: Rust backend utilizes Mozilla Fluent (`fluent-rs`) for handling complex pluralizations, variable interpolations, and notification templates.

---

## 4. Client Integration & API Contract Strategy

### 4.1 REST-First Philosophy & "Zero-SDK" Autonomy
A fundamental architectural tenet of Foundry is **REST-First and Zero-SDK Autonomy**:
1. **Self-Contained RESTful APIs**: All Foundry features (Dynamic CRUD, System Configs, Sub-System Custom APIs) are exposed as clean, predictable, standard HTTP RESTful endpoints.
2. **Zero Vendor Lock-In**: Client applications (Web, iOS, Android, Desktop, Mini-Programs, Backend Microservices) **do not require an official SDK** to integrate with Foundry. Any standard HTTP client (`fetch`, `axios`, `Ktor`, `Retrofit`, `cURL`, `reqwest`) can directly consume the APIs.
3. **Core Repository Decoupling**: Client SDKs do not belong to the Foundry core repository. Keeping the core monorepo strictly focused on engine services, admin UI, and CLI ensures maximum architectural clarity, minimal CI maintenance overhead, and language neutrality.

```
+─────────────────────────────────────────────────────────────────────────────────────────+
|                                    CLIENT INTEGRATION MODES                             |
|                                                                                         |
|  [ Approach A: Direct REST / HTTP ] (Recommended / Native)                              |
|    Any App ──(HTTP JSON / Headers)──> Foundry REST Endpoints (/api/v1/s/:slug/...)       |
|                                                                                         |
|  [ Approach B: Generated Typed Client via OpenAPI ]                                     |
|    OpenAPI Spec (/api/v1/openapi.json) ──> OpenAPI Generator ──> Typed Client Code      |
|                                                                                         |
|  [ Approach C: External / Community SDKs ] (Out-of-Tree Repositories)                   |
|    Separate community-maintained repositories (e.g. foundkit/foundry-sdk-*)             |
+─────────────────────────────────────────────────────────────────────────────────────────+
```

### 4.2 OpenAPI-Native Integration & Type Generation

Foundry provides compile-time generated, real-time OpenAPI 3.0 specifications via `utoipa`:
- **Live Specification Endpoint**: The Foundry core server automatically exposes an up-to-date OpenAPI schema at `/api/v1/openapi.json` and interactive docs at `/api/v1/docs`.
- **Client Code Generation Workflow**:
  - Developers can feed `/api/v1/openapi.json` directly into standard code generators (e.g., `openapi-generator-cli`, `orval`, `hey-api`, or `kmp-openapi`) to produce typed clients on-demand for any language or framework (TypeScript, Kotlin, Swift, Go, Python, Dart/Flutter).
  - This eliminates client-server contract drift while completely removing the need for manual SDK maintenance inside the core system.

### 4.3 Unified Core Release Train & Versioning Model

Foundry adopts a **Synchronous Core Release Train** for all first-party components maintained in the monorepo:

```
+───────────────────────────────────────────────────────────────────────────────────────────+
|                               UNIFIED CORE RELEASE TRAIN                                  |
|                                                                                           |
|                           Foundry Core Engine (Rust Crates)                               |
|                                        v1.0.0                                             |
|                                          │                                                |
|                   ┌──────────────────────┼──────────────────────┐                         |
|                   ▼                      ▼                      ▼                         |
|             Foundry Server           Admin SPA             Foundry CLI                    |
|             (apps/server)           (apps/admin)            (apps/cli)                    |
|                 v1.0.0                 v1.0.0                 v1.0.0                      |
|                   │                      │                      │                         |
|                   └──────────────────────┼──────────────────────┘                         |
|                                          ▼                                                |
|                                OpenAPI 3.0 Specification                                  |
|                                  (/api/v1/openapi.json)                                   |
+───────────────────────────────────────────────────────────────────────────────────────────+
```

1. **Version Parity (严格版本对齐)**:
   - All core applications (`apps/server`, `apps/admin`, `apps/cli`) share identical version tags `vX.Y.Z`.
   - The OpenAPI specification version strictly reflects the corresponding server version.
2. **Semantic Versioning Specification (SemVer 2.0.0)**:
   - **Major (`X.0.0`)**: Breaking changes to RESTful API structures or major engine architectural shifts.
   - **Minor (`X.Y.0`)**: New backward-compatible sub-system features, new platform primitives, or non-breaking API additions.
   - **Patch (`X.Y.Z`)**: Backward-compatible bug fixes, security patches, and performance optimizations.

---

## 5. Technology Stack Reference Matrix

| Layer | Technology Choice | Description / Rationale |
| :--- | :--- | :--- |
| **Monorepo Manager** | Cargo Workspaces + npm / pnpm | Dual setup managing Rust workspace crates, `systems/` custom packages, & `apps/admin` |
| **Backend Language & Runtime** | Rust (2024 Edition / 1.85+) + Tokio Async | Maximum performance, low memory footprint, zero-cost abstractions |
| **HTTP Web Framework** | Axum 0.8+ | High-performance, ergonomic routing, modular middleware architecture |
| **Subsystem Code Modularization** | `SubsystemModule` Trait + `systems/src/*` | Modular controller, logic, and DTO layers scoped per unique `system_slug` |
| **Validation & Serialization** | `validator` + `serde` / `serde_json` | Declarative request payload validation and high-performance serialization |
| **Primary Database & ORM** | PostgreSQL 18.6+ & SQLx | Dynamic JSONB, Zero-DDL storage, GIN indexing, ACID transactions |
| **Cache & Distributed State** | Redis 8+ | Tenant-namespaced caching, distributed locking, rate limiting |
| **Backend i18n & Localization**| `fluent-rs` / `unic-langid` | Mozilla Fluent format for localized error envelopes & templates |
| **Extension Engine** | Wasmtime (WebAssembly Runtime) | Sandboxed dynamic plugin execution without engine recompilation |
| **OpenAPI & API Contract** | Utoipa + Swagger UI | Rust compile-time OpenAPI 3.0 generation for both Auto-CRUD & Custom Controllers |
| **Admin Frontend SPA** | React 18+ / 19 / TypeScript 5+ / Vite 6+ / Tailwind CSS | Modern, accessible, responsive component-driven interface |
| **Frontend i18n Engine** | `react-i18next` + `i18next-browser-languagedetector` | Namespace-based translations with community pack hot-reloading |
| **API Client Tooling** | OpenAPI 3.0 (`openapi-generator`, `orval`, `hey-api`) | Type-safe client code generation without in-repo SDK lock-in |

---

## 6. Security, IAM & Isolation Model

1. **Strict Tenant Boundaries**: Axum middleware enforces `SystemContext` extraction before any handler is executed. All database operations and cache keys are strictly scoped to the active tenant.
2. **Admin IAM & Topic-Scoped RBAC**:
   - Super Admin (`super_admin`) holds universal permissions (`["*"]`) to manage the platform and grant permissions.
   - Normal Admins (`admin`) are constrained to specific sub-systems (`allowed_systems`).
3. **Non-GET Request Audit Trail**: Every write operation (POST, PUT, PATCH, DELETE, login) executed by administrators is automatically captured with operator identity, path, dynamic action name, parameters, IP, status, and latency.
4. **Fine-Grained Model Access Control**: Each dynamic model has visual granular toggles (`Public Read`, `Authenticated Read`, `Public Write`, `Private`). Role permissions are evaluated at the middleware level.
5. **Sandboxed Plugin Safety**: Dynamic WASM extensions run in an isolated memory space via `wasmtime` with zero access to the host filesystem or network unless explicitly whitelisted.

---

## 7. Operational & Deployment Architecture

- **Single Container Deployment**: A multi-stage `Dockerfile` compiles the Rust server binary and builds the Admin SPA assets into a single lightweight container image (under 50MB runtime footprint).
- **Embedded Asset Serving**: In self-hosted single-binary mode, the Axum server embeds the Admin SPA static assets directly, allowing users to run the entire platform with one binary and one command.
- **External Dependencies**: Requires only PostgreSQL (18.6+) and Redis (8+), easily orchestrated via `docker-compose.yml`.

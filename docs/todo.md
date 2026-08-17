# Foundry Development Roadmap & TODO List

> **Project**: `foundry`  
> **Organization**: [foundkit](https://github.com/foundkit)  
> **Repository**: Monorepo (Rust Engine + Sub-Systems Workspace + Web Admin UI + CLI)

---

## 📌 Legend & Status Tracker
- [ ] **Pending**
- [/] **In Progress**
- [x] **Completed**

---

## Milestone 0: Monorepo Foundation & Workspace Setup

- [x] **0.1 Repository & Project Scaffolding**
  - [x] Initialize Cargo Workspace for Rust backend crates and `systems/` workspace (`Cargo.toml`).
  - [x] Apply baseline database schema migration from `migrations/init.sql` (Mode A: Zero-DDL tables `systems`, `system_configs`, `models`, `model_fields`, `model_records`, `admins`, `audit_logs`).
  - [x] Initialize frontend project for Admin SPA in `apps/admin` (React + Vite + TypeScript + Tailwind CSS).
  - [x] Configure GitHub Actions CI for linting, testing, and formatting (`cargo fmt`, `cargo clippy`, frontend linting).
  - [x] Setup `docker/` container configurations (`docker-compose.yml` for PostgreSQL 18.6+ + Redis 8+).

- [x] **0.2 Directory Layout Creation**
  - [x] Create `apps/server` (Rust Axum binary entry point).
  - [x] Create `apps/admin` (Vite + React + Tailwind CSS Admin SPA with `src/components/` and `src/locales/`).
  - [x] Create `apps/cli` (Foundry CLI utility binary for system scaffolding and migrations).
  - [x] Create `systems/` (Sub-System custom code workspace with `systems/src/lib.rs` registry).
  - [x] Create core Rust crates (`crates/foundry_core`, `crates/foundry_storage`, `crates/foundry_auth`, `crates/foundry_engine`, `crates/foundry_extension`).

---

## Milestone 1: Core Engine & Multi-System Isolation (Backend MVP)

- [x] **1.1 System Context & Tenant Routing (`crates/foundry_core` & `crates/foundry_engine`)**
  - [x] Implement Sub-System metadata primitives: `system_id` (UUID) and immutable unique `system_slug` (`^[a-z0-9_-]{2,32}$`).
  - [x] Build Axum middleware to extract and validate `SystemContext` via URL path (`/api/v1/s/{system_slug}/...`), header (`X-Foundry-System-ID`), or subdomain.
  - [x] Parse `Accept-Language` header into `SystemContext.locale` for localized response handling.
  - [x] Implement global error handling with i18n JSON envelope (`code`, `message`, `i18n_key`, `args`).

- [x] **1.2 Zero-DDL Storage Engine & Model Runtime (`crates/foundry_storage`)**
  - [x] Implement System Configs engine (`system_configs`): single-row property management with visual form types, automatic aggregation into a single JSON object.
  - [x] Implement Dynamic Data Models engine (`models` & `model_fields`): schema metadata, field types (`string`, `richtext`, `image`, `file`, `integer`, `number`, `boolean`, `datetime`, `array`, `relation`).
  - [x] Build Data Model runtime ORM on `model_records` (`ctx.model("{model_slug}")`) with pagination, dynamic GIN JSON filtering, sorting, and atomic mutations.
  - [x] Implement programmatic Configs API (`ctx.configs()`) for easy retrieval of typed settings in custom logic.
  - [x] Implement in-memory schema validation (validating types, ranges, regex, required fields before database writes).
  - [x] Integrate Redis connection pool with strict tenant namespace key isolation (`foundry:{system_slug}:*`).

- [x] **1.3 Auto-CRUD & System Configs API Engine (`crates/foundry_engine`)**
  - [x] Implement dynamic Data Model Auto-CRUD endpoints: `GET` (List with filter/sort/pagination), `POST`, `GET /:id`, `PUT/PATCH /:id`, `DELETE /:id`.
  - [x] Implement System Configs endpoints: `GET /api/v1/s/{slug}/configs` and `PUT /api/v1/s/{slug}/configs`.
  - [x] Build fine-grained API exposure controller (Public Read, Authenticated Read, Public Write, Private/Disabled).

- [x] **1.4 Admin IAM & Topic-Scoped RBAC (`crates/foundry_auth`)**
  - [x] Implement Admin identity schema and password hashing via Argon2id.
  - [x] Provision default Super Admin (`super_admin`) with wildcard permissions (`allowed_systems: ["*"]`).
  - [x] Implement Super Admin capability to create/manage normal administrators (`admin`) and grant topic permissions (`allowed_systems: ["carnival_2026", "vip_mall"]`).
  - [x] JWT / Token issuing, verification, and refresh mechanism with `admin_id`, `role`, and `allowed_systems` claims.
  - [x] Axum middleware for evaluating topic-scoped access permissions.

- [x] **1.5 Sub-System Code-First Custom Extensibility Framework (`systems/` & `crates/foundry_core`)**
  - [x] Define `SubsystemModule` trait in `crates/foundry_core` (declaring `slug()`, `display_name()`, `register_routes()`, `register_openapi()`).
  - [x] Build static/dynamic subsystem route merger in `foundry_engine` to mount custom controllers alongside Auto-CRUD routes under `/api/v1/s/{system_slug}/...`.
  - [x] Implement parameter extraction, DTO declarative validation (`validator`), and custom handler bindings.
  - [x] Establish standard Three-Layer sub-system template:
    - `systems/src/<system_slug>/controllers/` (Axum handlers + OpenAPI annotations)
    - `systems/src/<system_slug>/logic/` (Domain business services & transactions)
    - `systems/src/<system_slug>/dto/` (Request/Response DTOs & Validation rules)
  - [x] Create example built-in sub-system (e.g. `carnival_demo`) demonstrating autonomous domain logic and custom endpoint routing.

---

## Milestone 2: Web Admin Dashboard (`apps/admin`)

- [x] **2.1 Dashboard Infrastructure & i18n Integration**
  - [x] Setup React + Vite + Tailwind CSS design system.
  - [x] Integrate `react-i18next` with built-in `en-US` and `zh-CN` locale bundles.
  - [x] Build UI language switcher & dynamic locale loading mechanism.
  - [x] Build Admin Login, Token Storage, and System Switcher header component (filtered by `allowed_systems`).
  - [x] Build responsive layout sidebar and navigation shell.

- [x] **2.2 System Management & Zero-DDL Schema Builder**
  - [x] **System Manager**: Visual UI for Super Admin to create, configure, and manage sub-systems with unique `system_slug` validation.
  - [x] **System Configs Editor (专题配置)**: Visual page to configure system-level properties (Banner, rules, times, toggles) with rich widgets (No raw JSON).
  - [x] **Data Model Builder (数据模型)**: Form UI to dynamically create multi-record business models, add/edit fields (`string`, `richtext`, `image`, `file`, `integer`, `number`, `boolean`, `datetime`, `array`), and configure widget options.

- [x] **2.3 Data Explorer & Custom API Directory**
  - [x] **Data Model Explorer**: Generic Data Table supporting dynamic columns, sorting, searching, and pagination for any multi-record business model.
  - [x] Dynamic Record Create / Edit / Delete modal interfaces with field-type specific widgets.

- [x] **2.4 Admin Management & Audit Logs UI**
  - [x] **Admin IAM Manager**: UI for Super Admin to create/edit normal admins and assign specific sub-systems (`allowed_systems`).
  - [x] **Audit Log Explorer**: Real-time log table displaying Non-GET admin operations with filters (by admin, sub-system, method, path, time range) and discrete inspector tabs for Headers (JSON), Raw Query String (Text), and Raw Request Body (Text / JSON Formatter).

---

## Milestone 3: Non-GET Write Audit Engine & Operation Interception

- [x] **3.1 Non-GET Audit Interceptor Middleware (`crates/foundry_engine`)**
  - [x] Intercept all state-mutating HTTP methods (`POST`, `PUT`, `PATCH`, `DELETE`) and login endpoints.
  - [x] Capture operator details (`admin_id`, `admin_username`), target `system_slug`, HTTP method, URL path, client IP, User-Agent, status code, and latency (`duration_ms`).
  - [x] Extract and store discrete parameter columns: `headers` (sanitized header key-values JSONB), `query_params` (raw query string `VARCHAR(2048)`), and `body_params` (raw request body payload `TEXT`, supporting JSON, form-data, XML, etc.).
  - [x] Asynchronous non-blocking log write pipeline to PostgreSQL `audit_logs` table.

- [x] **3.2 Dynamic Action Name Mapping Registry**
  - [x] Implement flexible route-to-action-name resolver in application code (e.g., `/admin/auth/login` -> "管理员登录", `/admin/s/:slug/configs` -> "修改专题配置", `/admin/s/:slug/models` -> "创建数据模型").
  - [x] Provide simple programmatic registration API allowing custom sub-systems to register action names for their own non-GET endpoints.

- [x] **3.3 Sub-System Domain Autonomy Guidelines & Templates**
  - [x] Provide architectural best-practice guides for sub-systems implementing their own client user authentication (e.g. WeChat Mini-Program login, SMS OTP, OAuth2).
  - [x] Provide templates for sub-systems designing their own domain entities and transactional services.

---

## Milestone 4: Extension System, CLI Tooling, OpenAPI & Production Readiness

- [x] **4.1 Developer CLI Tooling (`apps/cli`)**
  - [x] `foundry-cli system new <system_slug> [--name <display_name>]`: Scaffolds a complete sub-system module in `systems/src/<system_slug>/` with `controllers/`, `logic/`, `dto/`, and `mod.rs`.
  - [x] `foundry-cli system list`: Lists all registered sub-systems, their slugs, and active endpoints.
  - [x] `foundry-cli admin create <username> <password>`: Provision administrators with Argon2id and topic assignment.
  - [x] `foundry-cli migrate`: Run zero-DDL baseline database migrations.

- [x] **4.2 Logic Extension Pipeline (`crates/foundry_extension`)**
  - [x] Implement Rust Trait hooks (`before_create`, `after_create`, `before_update`, `after_update`, `before_delete`, `after_delete`).

- [x] **4.3 Production Readiness & Deployment**
  - [x] Multi-stage Dockerfile bundling server binary and Admin UI SPA assets into a single lightweight container.
  - [x] Production docker-compose environment with PostgreSQL 18.6+ and Redis 8+.
  - [x] Comprehensive documentation in `README.md` and `docs/`.

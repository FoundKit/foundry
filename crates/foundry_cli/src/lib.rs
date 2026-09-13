use clap::{Parser, Subcommand};
use foundry_core::types::is_valid_slug;
use foundry_storage::{init_db_pool, run_migrations};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "foundry")]
#[command(
    about = "Foundry CLI - Scaffolding, migrations, and developer tooling for Foundry applications",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a brand new standalone Foundry application project
    New {
        /// Project directory name (e.g. my-app, blog-server)
        name: String,
        /// Optional path to local foundry crate (useful for local development or monorepos)
        #[arg(long)]
        path: Option<String>,
        /// Optional git repository URL (defaults to https://github.com/foundkit/foundry)
        #[arg(long)]
        git: Option<String>,
        /// Optional git branch (defaults to main)
        #[arg(long)]
        branch: Option<String>,
        /// Optional crates.io version override (e.g. 0.1.0)
        #[arg(long)]
        version: Option<String>,
    },
    /// Sub-system management and scaffolding within an existing application
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },
    /// Apply database migrations
    Migrate {
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Validate project structure and manifest integrity
    Validate {
        /// Target directory to validate (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Scaffold a new sub-system module inside `src/systems/<slug>/`
    New {
        /// Unique sub-system slug (e.g., blog, billing, orders)
        slug: String,
        /// Display name for the sub-system
        #[arg(short, long)]
        name: Option<String>,
        /// Target project root directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        project_dir: String,
    },
    /// List all discovered sub-systems in current project
    List {
        /// Target directory to scan
        #[arg(default_value = ".")]
        path: String,
    },
}

pub async fn run_cli() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            path,
            git,
            branch,
            version,
        } => {
            scaffold_project(
                &name,
                ProjectOptions {
                    local_path: path.as_deref(),
                    git: git.as_deref(),
                    branch: branch.as_deref(),
                    version: version.as_deref(),
                },
            )?;
        }
        Commands::System { action } => match action {
            SystemCommands::New {
                slug,
                name,
                project_dir,
            } => {
                scaffold_subsystem(&slug, name.as_deref(), &project_dir)?;
            }
            SystemCommands::List { path } => {
                list_subsystems(&path)?;
            }
        },
        Commands::Migrate { database_url } => {
            let db_url = database_url.unwrap_or_else(|| {
                std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
                })
            });
            println!("Connecting to database at {}...", db_url);
            let pool = init_db_pool(&db_url, 5).await?;

            let migrations_dir = Path::new("migrations");
            if migrations_dir.is_dir() {
                let mut entries: Vec<_> = fs::read_dir(migrations_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sql"))
                    .collect();
                entries.sort_by_key(|e| e.file_name());

                if !entries.is_empty() {
                    println!(
                        "Applying {} migration script(s) from migrations/:",
                        entries.len()
                    );
                    for entry in entries {
                        let path = entry.path();
                        println!("  • Applying {:?}...", path.file_name().unwrap_or_default());
                        let sql = fs::read_to_string(&path)?;
                        sqlx::raw_sql(&sql).execute(&pool).await?;
                    }
                    println!("✅ Project migrations applied successfully.");
                    return Ok(());
                }
            }

            run_migrations(&pool).await?;
            println!("✅ Database migrations applied successfully.");
        }
        Commands::Validate { path } => {
            validate_project(&path)?;
        }
    }

    Ok(())
}

/// Project scaffolding options
#[derive(Debug, Clone, Default)]
pub struct ProjectOptions<'a> {
    pub local_path: Option<&'a str>,
    pub git: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub version: Option<&'a str>,
}

/// Create a new standalone user application
pub fn scaffold_project(name: &str, opts: ProjectOptions<'_>) -> anyhow::Result<PathBuf> {
    let project_dir = PathBuf::from(name);
    let pkg_name = project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{:?}' already exists.", project_dir);
    }

    println!("🚀 Creating new Foundry application: {}...", name);

    fs::create_dir_all(project_dir.join("src/systems/sample/controllers"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/logic"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/dto"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/custom_pages"))?;
    fs::create_dir_all(project_dir.join("migrations"))?;
    fs::write(
        project_dir.join("migrations/init.sql"),
        include_str!("../../foundry_storage/migrations/init.sql"),
    )?;
    fs::create_dir_all(project_dir.join("config"))?;
    fs::create_dir_all(project_dir.join("dev"))?;

    // 1. Cargo.toml
    let foundry_dep = if let Some(p) = opts.local_path {
        format!(r#"foundry = {{ path = "{}" }}"#, p)
    } else if let Some(v) = opts.version {
        format!(r#"foundry = "{}"#, v)
    } else {
        let repo = opts.git.unwrap_or("https://github.com/foundkit/foundry");
        let branch = opts.branch.unwrap_or("main");
        format!(r#"foundry = {{ git = "{}", branch = "{}" }}"#, repo, branch)
    };

    let cargo_toml = format!(
        r#"[package]
name = "{pkg_name}"
version = "0.1.0"
edition = "2024"
authors = ["Your Name <you@example.com>"]

[dependencies]
{foundry_dep}
tokio = {{ version = "1.44", features = ["full"] }}
axum = {{ version = "0.8", features = ["macros"] }}
tower = {{ version = "0.5", features = ["util"] }}
tower-http = {{ version = "0.6", features = ["cors", "trace", "fs"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
validator = {{ version = "0.20", features = ["derive"] }}
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
"#,
        pkg_name = pkg_name,
        foundry_dep = foundry_dep
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // 2. src/main.rs
    let main_rs = r#"pub mod systems;

use foundry::prelude::*;
use systems::SampleSubsystem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load configuration from environment variables (.env)
    let config = FoundryConfig::from_env();

    // 2. Build Foundry Application instance with registered subsystems
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(SampleSubsystem)
        .build()
        .await?;

    // 3. Start server
    app.run().await?;
    Ok(())
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // 3. src/systems/mod.rs
    let systems_mod = "pub mod sample;\npub use sample::SampleSubsystem;\n";
    fs::write(project_dir.join("src/systems/mod.rs"), systems_mod)?;

    // 4. Sample Subsystem: DTO
    let sample_dto = r#"use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GreetRequest {
    #[validate(length(min = 1, max = 50, message = "Name must be between 1 and 50 characters"))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct GreetResponse {
    pub message: String,
}
"#;
    fs::write(
        project_dir.join("src/systems/sample/dto/mod.rs"),
        sample_dto,
    )?;

    // 5. Sample Subsystem: Logic
    let sample_logic = r#"use crate::systems::sample::dto::{GreetRequest, GreetResponse};
use foundry::prelude::*;

pub struct SampleService;

impl SampleService {
    pub async fn greet(_ctx: &SystemContext, req: GreetRequest) -> AppResult<GreetResponse> {
        Ok(GreetResponse {
            message: format!("Hello, {}! Welcome to Foundry Framework.", req.name),
        })
    }
}
"#;
    fs::write(
        project_dir.join("src/systems/sample/logic/mod.rs"),
        sample_logic,
    )?;

    // 6. Sample Subsystem: Controllers
    let sample_controller = r#"use axum::{extract::Extension, routing::post, Json, Router};
use foundry::prelude::*;
use validator::Validate;

use crate::systems::sample::dto::{GreetRequest, GreetResponse};
use crate::systems::sample::logic::SampleService;

pub fn build_routes() -> Router {
    Router::new().route("/greet", post(handle_greet))
}

pub async fn handle_greet(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<GreetRequest>,
) -> AppResult<Json<ApiResponse<GreetResponse>>> {
    payload.validate()?;
    let res = SampleService::greet(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(res)))
}
"#;
    fs::write(
        project_dir.join("src/systems/sample/controllers/mod.rs"),
        sample_controller,
    )?;

    // 7. Sample Subsystem: Custom Admin Page
    let custom_page = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Sample Dashboard</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 min-h-screen p-6">
  <div class="max-w-4xl mx-auto space-y-4">
    <h1 class="text-xl font-bold text-indigo-600 dark:text-indigo-400">📊 Sample Subsystem Dashboard</h1>
    <p class="text-xs text-slate-500">Custom Admin UI Extension embedded seamlessly in Foundry Admin Shell.</p>
  </div>
  <script>
    window.addEventListener('message', function(event) {
      if (event.data?.type === 'FOUNDRY_INIT') {
        const p = event.data.payload;
        if (p.theme === 'dark') document.documentElement.classList.add('dark');
      }
    });
  </script>
</body>
</html>
"#;
    fs::write(
        project_dir.join("src/systems/sample/custom_pages/overview.html"),
        custom_page,
    )?;

    // 8. Sample Subsystem: mod.rs
    let sample_mod = r#"pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub struct SampleSubsystem;

impl SubsystemModule for SampleSubsystem {
    fn slug(&self) -> &'static str {
        "sample"
    }

    fn display_name(&self) -> &'static str {
        "Sample Subsystem"
    }

    fn description(&self) -> &'static str {
        "A starter sample subsystem demonstrating routes, services, and admin extensions"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());
        let possible_dirs = [
            PathBuf::from("src/systems/sample/custom_pages"),
            PathBuf::from("static/custom_pages/sample"),
        ];
        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "overview".to_string(),
            title: "Sample Overview".to_string(),
            icon: "LayoutDashboard".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/sample/ext/custom-pages/overview.html".to_string(),
            required_role: None,
        }]
    }
}
"#;
    fs::write(project_dir.join("src/systems/sample/mod.rs"), sample_mod)?;

    // 9. .env.example
    let env_example = r#"HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
# Auto-migrate database schema on server startup (true: automatic sync; false: strict manual DBA control)
AUTO_MIGRATE=true
"#;
    fs::write(project_dir.join(".env.example"), env_example)?;
    fs::write(project_dir.join(".env"), env_example)?;

    // 10. dev/docker-compose.yml
    let docker_compose = format!(
        r#"services:
  postgres:
    image: postgres:18-alpine
    container_name: {pkg_name}-postgres
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgrespassword
      POSTGRES_DB: foundry
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql
      - ../migrations/init.sql:/docker-entrypoint-initdb.d/01_init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: {pkg_name}-redis
    restart: unless-stopped
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
"#,
        pkg_name = pkg_name
    );
    fs::write(project_dir.join("dev/docker-compose.yml"), docker_compose)?;

    // 10.1 dev/init-db.sh
    let init_db_sh = r#"#!/usr/bin/env bash
# ==============================================================================
# Local Development Database Initialization & Reset Script
# ==============================================================================
set -e

DEV_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$DEV_DIR/.." && pwd)"
INIT_SQL="$PROJECT_ROOT/migrations/init.sql"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgrespassword}"
DB_NAME="${DB_NAME:-foundry}"

echo "🚀 Initializing Foundry Development Database ($DB_NAME)..."

if [ ! -f "$INIT_SQL" ]; then
    echo "❌ Error: $INIT_SQL not found!"
    exit 1
fi

if command -v docker &>/dev/null && docker compose -f "$DEV_DIR/docker-compose.yml" ps --status running | grep -q "postgres"; then
    echo "📦 Applying migrations/init.sql via docker compose exec..."
    docker compose -f "$DEV_DIR/docker-compose.yml" exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" -f /docker-entrypoint-initdb.d/01_init.sql
elif command -v psql &>/dev/null; then
    echo "🐘 Applying migrations/init.sql via local psql..."
    PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$INIT_SQL"
else
    echo "⚠️ Neither active docker container nor local psql CLI found."
    echo "Please ensure the database container is running: docker compose -f dev/docker-compose.yml up -d"
    exit 1
fi

echo "✅ Development database initialized successfully!"
"#;
    fs::write(project_dir.join("dev/init-db.sh"), init_db_sh)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(
            project_dir.join("dev/init-db.sh"),
            fs::Permissions::from_mode(0o755),
        );
    }

    // 10.2 dev/init-dev-db.sql
    let init_dev_db_sql = r#"-- ==============================================================================
-- Foundry Development Database Bootstrap Script
-- Usage with local PostgreSQL superuser:
--   psql -U postgres -f dev/init-dev-db.sql
-- ==============================================================================

SELECT 'CREATE DATABASE foundry'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'foundry')\gexec

\c foundry

\i ../migrations/init.sql
"#;
    fs::write(project_dir.join("dev/init-dev-db.sql"), init_dev_db_sql)?;

    // 11. .gitignore
    let gitignore = r#"/target
.env
*.log
.DS_Store

# Local development resources
dev/
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    // 12. README.md
    let readme = format!(
        r#"# {name}

基于 [Foundry](https://github.com/foundkit/foundry) 现代模块化后端框架构建的独立业务应用。

---

## ⚡ 快速上手

### 1. 启动基础依赖（PostgreSQL & Redis）

项目在 `dev/` 目录中预置了本地开发所需的容器编排配置：

```bash
# 启动本地 PostgreSQL 18 与 Redis 7 容器
docker compose -f dev/docker-compose.yml up -d
```

**服务与默认连接信息（与 `.env` 保持一致）：**

| 服务 | 连接地址 | 说明 |
| :--- | :--- | :--- |
| **PostgreSQL 18** | `postgres://postgres:postgrespassword@localhost:5432/foundry` | 首次启动自动执行 `migrations/init.sql` 初始化库表与超管 |
| **Redis 7** | `redis://127.0.0.1:6379` | 提供系统缓存与会话存储 |

> **数据库运维与管控说明**：
> - **开发库重置**：执行 `bash dev/init-db.sh` 快速重置开发库并重新导入初始脚本；或执行 `docker compose -f dev/docker-compose.yml down -v` 清空数据卷。
> - **DBA 手动导入（严格管控）**：支持直接通过原生命令导入：`psql -h 127.0.0.1 -U postgres -d foundry -f migrations/init.sql`。
> - **发布管控**：在 `.env` 中设置 `AUTO_MIGRATE=false` 可关闭后端启动时的自动迁移检测，实现严格的手动发布审计。

---

### 2. 启动应用服务

```bash
cargo run
```

服务启动后默认监听 `http://localhost:8080`：

| 模块 / 接口 | 地址 / 凭证 | 说明 |
| :--- | :--- | :--- |
| **管理后台** | `http://localhost:8080/admin/` | 内置开箱即用 React 控制台（访问 `/` 自动重定向） |
| **超管凭证** | `admin` / `admin123456` | 默认系统管理员账号与初始密码 |
| **健康检查** | `GET http://localhost:8080/api/v1/health` | 状态探针（健康返回 `OK`） |
| **API 根路径** | `http://localhost:8080/api/v1` | 业务子系统 RESTful API 入口 |

---

## 🧩 业务子系统开发

### 架构原理：单体模块化 (Modular Monolith)

Foundry 采用**单体模块化**架构，所有子系统（Subsystem）均为工程内部独立的业务领域模块：
* **单二进制与同一进程**：所有子系统随应用统一编译为一个二进制程序，共享数据库连接池与 Redis 实例，监听统一端口，**无需拆分微服务，绝不需要单独部署**。
* **统一自动路由**：子系统注册后，框架自动将其 API 挂载至 `/api/v1/s/{{system_slug}}/ext/*`，并自动将自定义管理看板嵌入管理后台侧边栏。
* **数据逻辑隔离**：Zero-DDL 动态模型与数据均以 `system_slug` 自动划分逻辑边界，既保证业务自治高内聚，又保持单体应用极简的开发和运维体验。

---

### 开发流程

#### 1. 创建子系统代码骨架
通过 CLI 在 `src/systems/<slug>/` 下一键生成包含完整三层架构的骨架：
```bash
foundry system new billing --name "账单与支付中心"
```
生成的文件职责清晰：
* `dto/mod.rs`：请求入参与校验规则（基于 `validator` 注解）
* `logic/mod.rs`：领域业务逻辑与数据库读写（注入 `SystemContext` 与 `DbPool`）
* `controllers/mod.rs`：Axum HTTP 控制器路由
* `custom_pages/`：嵌入管理后台的专属业务运营看板（HTML/React）

#### 2. 注册并启用
在代码中两步将新子系统装配到应用主进程中：
1. **导出模块** (`src/systems/mod.rs`)：
   ```rust
   pub mod billing;
   pub use billing::BillingSubsystem;
   ```
2. **注册到 App 构建器** (`src/main.rs`)：
   ```rust
   let app = FoundryApp::builder()
       .config(config)
       .register_subsystem(SampleSubsystem)
       .register_subsystem(BillingSubsystem) // 注册新子系统
       .build()
       .await?;
   ```

> 💡 **提示**：执行 `foundry system list` 可随时查看当前已识别注册的所有子系统。

---

## 🗄️ 数据存储与模型开发

Foundry 提供两种开发范式，可按业务场景自由组合：

### 模式 A：Zero-DDL 动态模型（免表迁移、即开即用）
适用于中后台配置、内容发布、快速迭代原型。无需手写 DDL 建表，数据按子系统自动隔离并自动提供 RESTful 接口。

```rust
use foundry_storage::models::RecordStore;
use serde_json::json;

// 写入数据（自动按子系统与模型 slug 隔离归集）
let record = RecordStore::create(&db, &ctx.system_slug, "articles", json!({{
    "title": "Hello Foundry",
    "content": "免 DDL 快速存储",
    "views": 100
}})).await?;

// 自动生成 RESTful 接口：
// GET/POST       /api/v1/s/{{system_slug}}/{{model_slug}}
// GET/PUT/DELETE /api/v1/s/{{system_slug}}/{{model_slug}}/{{id}}
```
*提示：可在管理后台（`/admin`）可视化添加模型定义与动态字段。*

### 模式 B：原生 SQL 迁移与强类型模型（高并发、事务、复杂关联）
适用于核心交易、复杂多表 Join、高性能严谨业务。

1. **编写 SQL 迁移**：在 `migrations/` 下新增脚本（如 `001_create_articles.sql`）：
   ```sql
   CREATE TABLE IF NOT EXISTS articles (
       id BIGSERIAL PRIMARY KEY,
       system_slug VARCHAR(64) NOT NULL,
       title VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   CREATE INDEX IF NOT EXISTS idx_articles_system_slug ON articles(system_slug);
   ```
2. **应用迁移**：
   ```bash
   foundry migrate
   # 注：若 .env 中 AUTO_MIGRATE=true，应用启动时亦会自动检测并应用
   ```
3. **强类型查询（SQLx）**：
   ```rust
   #[derive(Debug, sqlx::FromRow, serde::Serialize)]
   pub struct Article {{
       pub id: i64,
       pub system_slug: String,
       pub title: String,
   }}

   let rows = sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE system_slug = $1")
       .bind(&ctx.system_slug)
       .fetch_all(&db)
       .await?;
   ```

---

## 🛠️ 常用开发 CLI 命令

CLI 工具专注于辅助代码开发、骨架生成与工程规范校验：

```bash
# 1. 创建业务子系统代码骨架
foundry system new <slug> --name "<显示名称>"

# 2. 查看当前工程所有已注册子系统
foundry system list

# 3. 手动执行数据库迁移
foundry migrate

# 4. 校验当前工程规范与子系统目录结构
foundry validate
```

> 💡 **提示**：管理员账号增删、角色分配及权限管控，直接登录管理后台（`/admin`）进行可视化操作即可，无需在终端处理。
"#,
        name = name
    );
    fs::write(project_dir.join("README.md"), readme)?;

    println!(
        "✅ Project '{}' created successfully at {:?}!",
        name, project_dir
    );
    println!("👉 Next steps:");
    println!("   cd {}", name);
    println!("   docker compose -f dev/docker-compose.yml up -d");
    println!("   cargo run");

    Ok(project_dir)
}

/// Scaffold a new subsystem in an existing project
pub fn scaffold_subsystem(slug: &str, name: Option<&str>, project_dir: &str) -> anyhow::Result<()> {
    if !is_valid_slug(slug, 32) {
        anyhow::bail!(
            "Invalid system slug: '{}'. Must be 2-32 lowercase alphanumeric characters or underscore/hyphen.",
            slug
        );
    }

    let display_name = name.unwrap_or(slug);
    let base_dir = Path::new(project_dir).join("src/systems").join(slug);

    if base_dir.exists() {
        anyhow::bail!("Subsystem directory already exists at {:?}", base_dir);
    }

    println!(
        "🚀 Scaffolding new subsystem '{}' ({}) at {:?}...",
        slug, display_name, base_dir
    );

    fs::create_dir_all(base_dir.join("controllers"))?;
    fs::create_dir_all(base_dir.join("logic"))?;
    fs::create_dir_all(base_dir.join("dto"))?;
    fs::create_dir_all(base_dir.join("custom_pages"))?;

    // DTO
    fs::write(
        base_dir.join("dto/mod.rs"),
        r#"use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct SampleRequest {
    #[validate(length(min = 1))]
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SampleResponse {
    pub message: String,
}
"#,
    )?;

    // Logic
    fs::write(
        base_dir.join("logic/mod.rs"),
        format!(
            r#"use crate::systems::{slug}::dto::{{SampleRequest, SampleResponse}};
use foundry::prelude::*;

pub struct SampleService;

impl SampleService {{
    pub async fn execute(_ctx: &SystemContext, req: SampleRequest) -> AppResult<SampleResponse> {{
        Ok(SampleResponse {{
            message: format!("Processed request: {{}}", req.query),
        }})
    }}
}}
"#,
            slug = slug
        ),
    )?;

    // Controller
    fs::write(
        base_dir.join("controllers/mod.rs"),
        format!(
            r#"use axum::{{extract::Extension, routing::post, Json, Router}};
use foundry::prelude::*;
use validator::Validate;

use crate::systems::{slug}::dto::{{SampleRequest, SampleResponse}};
use crate::systems::{slug}::logic::SampleService;

pub fn build_routes() -> Router {{
    Router::new().route("/sample", post(handle_sample))
}}

pub async fn handle_sample(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<SampleRequest>,
) -> AppResult<Json<ApiResponse<SampleResponse>>> {{
    payload.validate()?;
    let result = SampleService::execute(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}}
"#,
            slug = slug
        ),
    )?;

    // Custom Page HTML
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{display_name} Admin</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 min-h-screen p-6">
  <div class="max-w-4xl mx-auto space-y-4">
    <h1 class="text-xl font-bold text-indigo-600 dark:text-indigo-400">📊 {display_name} Dashboard</h1>
    <p class="text-xs text-slate-500">Subsystem slug: <code class="font-mono font-semibold">{slug}</code></p>
  </div>
  <script>
    window.addEventListener('message', function(event) {{
      if (event.data?.type === 'FOUNDRY_INIT') {{
        const p = event.data.payload;
        if (p.theme === 'dark') document.documentElement.classList.add('dark');
      }}
    }});
  </script>
</body>
</html>"#,
        display_name = display_name,
        slug = slug
    );
    fs::write(base_dir.join("custom_pages/overview.html"), html_content)?;

    // Subsystem Module struct
    let struct_name = format!(
        "{}Subsystem",
        slug.split(['_', '-'])
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<String>()
    );

    let mod_content = format!(
        r#"pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub struct {struct_name};

impl SubsystemModule for {struct_name} {{
    fn slug(&self) -> &'static str {{
        "{slug}"
    }}

    fn display_name(&self) -> &'static str {{
        "{display_name}"
    }}

    fn register_routes(&self, router: Router) -> Router {{
        let mut r = router.merge(controllers::build_routes());
        let possible_dirs = [
            PathBuf::from("src/systems/{slug}/custom_pages"),
            PathBuf::from("static/custom_pages/{slug}"),
        ];
        for dir in possible_dirs {{
            if dir.exists() {{
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }}
        }}
        r
    }}

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {{
        vec![
            CustomAdminPageSpec {{
                key: "{slug}_overview".to_string(),
                title: "{display_name} Overview".to_string(),
                icon: "LayoutDashboard".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/{slug}/ext/custom-pages/overview.html".to_string(),
                required_role: None,
            }},
        ]
    }}
}}
"#,
        struct_name = struct_name,
        slug = slug,
        display_name = display_name
    );
    fs::write(base_dir.join("mod.rs"), mod_content)?;

    println!("✅ Subsystem '{}' created at {:?}", slug, base_dir);
    println!(
        "👉 Next step: Export and register `pub mod {};` in `src/systems/mod.rs` and add `.register_subsystem({}::{})` in `src/main.rs`.",
        slug, slug, struct_name
    );

    Ok(())
}

fn list_subsystems(project_path: &str) -> anyhow::Result<()> {
    let p = Path::new(project_path);
    println!("📦 Discovered Subsystems in {:?}", p);

    let systems_dir = p.join("src/systems");
    if let Ok(entries) = fs::read_dir(systems_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                println!("  • {} (Subsystem)", name);
            }
        }
    }

    Ok(())
}

fn validate_project(path: &str) -> anyhow::Result<()> {
    let p = Path::new(path);
    println!("🔍 Validating Foundry Project at {:?}", p);

    let cargo_toml = p.join("Cargo.toml");
    let main_rs = p.join("src/main.rs");
    let dev_compose = p.join("dev/docker-compose.yml");

    if !cargo_toml.exists() {
        println!("  ⚠️ Warning: Cargo.toml not found at {:?}", cargo_toml);
    } else {
        println!("  ✅ Cargo.toml present");
    }

    if !main_rs.exists() {
        println!("  ⚠️ Warning: src/main.rs not found at {:?}", main_rs);
    } else {
        println!("  ✅ src/main.rs present");
    }

    if !dev_compose.exists() {
        println!("  ℹ️ Note: dev/docker-compose.yml not found (optional local dev environment)");
    } else {
        println!("  ✅ dev/docker-compose.yml present");
    }

    list_subsystems(path)?;
    println!("✅ Project validation complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_project_and_subsystem() {
        let temp_dir =
            std::env::temp_dir().join(format!("foundry_test_cli_{}", uuid::Uuid::new_v4()));
        let project_name = temp_dir.to_string_lossy().to_string();

        let path = scaffold_project(
            &project_name,
            ProjectOptions {
                local_path: Some("../../../crates/foundry"),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(path.join("Cargo.toml").exists());
        assert!(path.join("src/main.rs").exists());
        assert!(path.join("dev/docker-compose.yml").exists());
        let compose = fs::read_to_string(path.join("dev/docker-compose.yml")).unwrap();
        assert!(compose.contains("postgres:18-alpine"));
        assert!(compose.contains("postgres_data:/var/lib/postgresql\n"));
        assert!(path.join(".gitignore").exists());
        let gitignore = fs::read_to_string(path.join(".gitignore")).unwrap();
        assert!(gitignore.contains("dev/"));
        assert!(path.join("src/systems/sample/mod.rs").exists());
        assert!(path.join("src/systems/sample/controllers/mod.rs").exists());
        assert!(path.join("migrations/init.sql").exists());
        assert!(path.join("dev/init-db.sh").exists());
        assert!(path.join("dev/init-dev-db.sql").exists());
        assert!(compose.contains("01_init.sql"));

        // Scaffold additional subsystem
        scaffold_subsystem("orders", Some("Order Management"), &project_name).unwrap();
        assert!(path.join("src/systems/orders/mod.rs").exists());
        assert!(path.join("src/systems/orders/controllers/mod.rs").exists());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}

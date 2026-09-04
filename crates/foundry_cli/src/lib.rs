use clap::{Parser, Subcommand};
use foundry_auth::hash_password;
use foundry_core::types::is_valid_slug;
use foundry_storage::{AdminStore, init_db_pool, run_migrations};
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
    /// Administrator account management
    Admin {
        #[command(subcommand)]
        action: AdminCommands,
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
        /// Unique sub-system slug (e.g., blog, carnival_demo, vip_mall)
        slug: String,
        /// Display name for the sub-system
        #[arg(short, long)]
        name: Option<String>,
        /// Target project root directory (defaults to current directory)
        #[arg(long, default_value = ".")]
        project_dir: String,
    },
    /// Scaffold a standalone external file-based sub-system directory
    NewExternal {
        /// Unique sub-system slug
        slug: String,
        /// Display name for the sub-system
        #[arg(short, long)]
        name: Option<String>,
        /// Target directory path
        #[arg(long, default_value = "./external_systems")]
        target_dir: String,
    },
    /// List all discovered sub-systems in current project
    List {
        /// Target directory to scan
        #[arg(default_value = ".")]
        path: String,
    },
}

#[derive(Subcommand)]
enum AdminCommands {
    /// Create an administrator account
    Create {
        username: String,
        password: String,
        #[arg(short, long, default_value = "admin")]
        role: String,
        #[arg(short, long, default_value = "*")]
        allowed: String,
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Reset an administrator's password
    ResetPassword {
        username: String,
        new_password: String,
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
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
            SystemCommands::NewExternal {
                slug,
                name,
                target_dir,
            } => {
                scaffold_external_subsystem(&slug, name.as_deref(), &target_dir)?;
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
            run_migrations(&pool).await?;
            println!("✅ Database migrations applied successfully.");
        }
        Commands::Admin { action } => match action {
            AdminCommands::Create {
                username,
                password,
                role,
                allowed,
                database_url,
            } => {
                let db_url = database_url.unwrap_or_else(|| {
                    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                        "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
                    })
                });
                let pool = init_db_pool(&db_url, 5).await?;
                let pwd_hash = hash_password(&password)?;
                let allowed_list: Vec<String> =
                    allowed.split(',').map(|s| s.trim().to_string()).collect();
                let allowed_json = serde_json::to_value(allowed_list)?;

                let admin =
                    AdminStore::create(&pool, &username, None, &pwd_hash, &role, allowed_json)
                        .await?;
                println!(
                    "✅ Created admin user '{}' (ID: {}, Role: {})",
                    admin.username, admin.id, admin.role
                );
            }
            AdminCommands::ResetPassword {
                username,
                new_password,
                database_url,
            } => {
                let db_url = database_url.unwrap_or_else(|| {
                    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                        "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
                    })
                });
                let pool = init_db_pool(&db_url, 5).await?;
                let pwd_hash = hash_password(&new_password)?;
                sqlx::query(
                    "UPDATE admins SET password_hash = $1, updated_at = NOW() WHERE username = $2",
                )
                .bind(&pwd_hash)
                .bind(&username)
                .execute(&pool)
                .await?;
                println!("✅ Password for admin '{}' reset successfully.", username);
            }
        },
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
      - postgres_data:/var/lib/postgresql/data
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

### 1. 启动本地开发数据库
项目在 `dev/` 目录中预置了本地专用的 PostgreSQL 18 与 Redis 7 容器编排配置（已在 `.gitignore` 中配置忽略，专门用于存放本地开发资源，不污染代码仓库）：

```bash
# 启动本地开发数据库与缓存
docker compose -f dev/docker-compose.yml up -d

# 查看运行状态
docker compose -f dev/docker-compose.yml ps

# 停止数据库与缓存
docker compose -f dev/docker-compose.yml down
```

> **默认连接配置**（与 `.env` 保持一致）：
> - PostgreSQL: `postgres://postgres:postgrespassword@localhost:5432/foundry`
> - Redis: `redis://127.0.0.1:6379`

### 2. 运行应用服务
```bash
cargo run
```
服务启动后将监听 `http://localhost:8080`：
- 健康检查：`curl http://localhost:8080/api/v1/health`
- 管理后台：`http://localhost:8080/admin`（默认超管账号：`admin` / `admin123456`）

---

## 🧩 子项目 / 子系统管理

Foundry 采用高内聚的子系统架构，支持以下操作：

### 1. 创建代码优先子系统（内聚在工程内）
```bash
# 创建新子系统骨架（位于 src/systems/<slug>/）
foundry system new <slug> --name "子系统显示名称"
# 示例：
foundry system new billing --name "账单与支付中心"
```
**注册步骤**：
1. 在 `src/systems/mod.rs` 导出：
   ```rust
   pub mod billing;
   pub use billing::BillingSubsystem;
   ```
2. 在 `src/main.rs` 中注册到 App 构建器：
   ```rust
   let app = FoundryApp::builder()
       .config(config)
       .register_subsystem(SampleSubsystem)
       .register_subsystem(BillingSubsystem) // 新增注册
       .build()
       .await?;
   ```

### 2. 创建独立外部文件式子系统
```bash
# 生成包含 subsystem.json 与 custom_pages 的独立目录
foundry system new-external carnival --name "嘉年华运营活动"
```

### 3. 查看当前工程所有已识别子系统
```bash
foundry system list
```

---

## 🗄️ 数据模型创建与管理

Foundry 提供两种模型开发范式：

### 方式一：Zero-DDL 动态数据模型（免迁移、即写即用）
无需手写 SQL DDL 与数据库表变更，自动按 `system_slug` 和 `model_slug` 隔离，并自动提供 RESTful CRUD 接口。

- **代码中快速写入与读取**：
```rust
use foundry_storage::models::RecordStore;
use serde_json::json;

// 插入动态模型记录
let record = RecordStore::create(
    &db,
    &ctx.system_slug,  // 所属子系统 slug
    "articles",         // 模型 slug
    json!({{
        "title": "Hello Foundry",
        "content": "动态模型无需执行 DDL 迁移",
        "views": 100
    }})
).await?;

// 按 ID 查询
let item = RecordStore::get_by_id(&db, &ctx.system_slug, "articles", record.id).await?;

// 分页列表查询
let list = RecordStore::list(&db, &ctx.system_slug, "articles", 1, 20).await?;
```
- **自动 RESTful 接口**：
  - `GET/POST /api/v1/s/{{system_slug}}/{{model_slug}}`
  - `GET/PUT/DELETE /api/v1/s/{{system_slug}}/{{model_slug}}/{{id}}`
- **后台可视化配置**：
  可在 Admin 控制台 (`/admin`) 可视化添加模型定义与动态字段。

### 方式二：原生 SQL 迁移与强类型模型（高并发、事务、复杂 Join）
1. **编写迁移脚本**：在 `migrations/` 目录下添加 `001_create_articles.sql`：
   ```sql
   CREATE TABLE IF NOT EXISTS articles (
       id BIGSERIAL PRIMARY KEY,
       system_slug VARCHAR(64) NOT NULL,
       title VARCHAR(255) NOT NULL,
       content TEXT NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   CREATE INDEX IF NOT EXISTS idx_articles_system_slug ON articles(system_slug);
   ```
2. **应用迁移**：
   ```bash
   foundry migrate
   # 或在 .env 中设置 AUTO_MIGRATE=true，应用启动时会自动执行
   ```
3. **在子系统业务逻辑中使用 SQLx 强类型查询**：
   ```rust
   #[derive(Debug, sqlx::FromRow, serde::Serialize)]
   pub struct Article {{
       pub id: i64,
       pub system_slug: String,
       pub title: String,
       pub content: String,
   }}

   let rows = sqlx::query_as::<_, Article>(
       "SELECT * FROM articles WHERE system_slug = $1"
   )
   .bind(&ctx.system_slug)
   .fetch_all(&db)
   .await?;
   ```

---

## 🛠️ 常用 CLI 运维命令

```bash
# 1. 校验当前工程规范
foundry validate

# 2. 手动执行数据库迁移
foundry migrate

# 3. 创建管理员账号
foundry admin create --username admin --password secret --role super_admin

# 4. 重置管理员密码
foundry admin reset-password --username admin --new-password newsecret
```
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

/// Scaffold external standalone subsystem
pub fn scaffold_external_subsystem(
    slug: &str,
    name: Option<&str>,
    target_dir: &str,
) -> anyhow::Result<()> {
    if !is_valid_slug(slug, 32) {
        anyhow::bail!(
            "Invalid system slug: '{}'. Must be 2-32 lowercase alphanumeric characters or underscore/hyphen.",
            slug
        );
    }

    let display_name = name.unwrap_or(slug);
    let base_dir = Path::new(target_dir).join(slug);

    if base_dir.exists() {
        anyhow::bail!("Directory already exists at {:?}", base_dir);
    }

    fs::create_dir_all(base_dir.join("custom_pages"))?;

    let manifest = serde_json::json!({
        "slug": slug,
        "display_name": display_name,
        "description": format!("External subsystem {}", display_name),
        "version": "1.0.0",
        "custom_pages": [
            {
                "key": format!("{}_overview", slug),
                "title": format!("{} Overview", display_name),
                "icon": "LayoutDashboard",
                "type": "iframe",
                "entry": format!("/api/v1/s/{}/ext/custom-pages/overview.html", slug)
            }
        ]
    });
    fs::write(
        base_dir.join("subsystem.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{display_name}</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-slate-900 text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold text-indigo-400">{display_name} External View</h1>
  <p class="text-xs text-slate-400 mt-2">Slug: {slug}</p>
</body>
</html>"#,
        display_name = display_name,
        slug = slug
    );
    fs::write(base_dir.join("custom_pages/overview.html"), html)?;

    println!("✅ External subsystem '{}' created at {:?}", slug, base_dir);
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
                println!("  • {} (Code-first Subsystem)", name);
            }
        }
    }

    let external_dir = p.join("external_systems");
    if let Ok(entries) = fs::read_dir(external_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                println!("  • {} (External Standalone Subsystem)", name);
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
        assert!(path.join(".gitignore").exists());
        let gitignore = fs::read_to_string(path.join(".gitignore")).unwrap();
        assert!(gitignore.contains("dev/"));
        assert!(path.join("src/systems/sample/mod.rs").exists());
        assert!(path.join("src/systems/sample/controllers/mod.rs").exists());

        // Scaffold additional subsystem
        scaffold_subsystem("orders", Some("Order Management"), &project_name).unwrap();
        assert!(path.join("src/systems/orders/mod.rs").exists());
        assert!(path.join("src/systems/orders/controllers/mod.rs").exists());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}

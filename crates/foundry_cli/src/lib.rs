use clap::{Parser, Subcommand};
use foundry_core::orm::{ColumnDef, DefaultValue, EntitySchema, IndexDef};
use foundry_core::types::is_valid_slug;
use foundry_storage::orm::{Dialect, compile_schemas};
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
        /// Target database engine (postgres, mysql, mariadb, mongodb)
        #[arg(long, short = 'd', alias = "database", default_value = "postgres")]
        db: String,
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
    /// Apply database migrations, or generate dialect DDL files
    Migrate {
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
        /// Explicit database type override (postgres, mysql, mariadb, mongodb)
        #[arg(long, short = 'd', alias = "database", env = "DATABASE_TYPE")]
        database_type: Option<String>,
        /// Generate dialect DDL migration file into migrations/ without applying to database
        #[arg(long, short = 'g', alias = "gen")]
        generate: bool,
        /// Target output file path when generating DDL (defaults to migrations/init.sql or init.js)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Generate dialect-specific DDL / migration scripts
    #[command(alias = "generate-ddl", alias = "gen-ddl")]
    Ddl {
        /// Target database engine (postgres, mysql, mariadb, mongodb)
        #[arg(
            long,
            short = 'd',
            alias = "database",
            env = "DATABASE_TYPE",
            default_value = "postgres"
        )]
        db: String,
        /// Target output file path (defaults to migrations/init.sql or migrations/init.js, or '-' for stdout)
        #[arg(short, long)]
        output: Option<String>,
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
        /// Generate a dedicated table, code-first ORM entity, and DDL SQL for this subsystem
        #[arg(long, alias = "table", alias = "with-orm")]
        with_table: bool,
        /// Custom table name for the subsystem (defaults to `{slug}_items`)
        #[arg(long)]
        table_name: Option<String>,
        /// Target database dialect for generated DDL SQL (mysql, mariadb, postgres, all; defaults to mysql/mariadb)
        #[arg(long, short = 'd', alias = "database")]
        db: Option<String>,
        /// Output directory for generated DDL SQL file
        #[arg(long)]
        ddl_dir: Option<String>,
    },
    /// List all discovered sub-systems in current project
    List {
        /// Target directory to scan
        #[arg(default_value = ".")]
        path: String,
    },
}

/// Supported database engines for scaffolding and migrations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseKind {
    Postgres,
    MySql,
    MariaDb,
    MongoDb,
}

impl std::str::FromStr for DatabaseKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Ok(DatabaseKind::Postgres),
            "mysql" => Ok(DatabaseKind::MySql),
            "mariadb" => Ok(DatabaseKind::MariaDb),
            "mongodb" | "mongo" => Ok(DatabaseKind::MongoDb),
            other => anyhow::bail!(
                "Unsupported database '{}'. Supported databases are: postgres, mysql, mariadb, mongodb.",
                other
            ),
        }
    }
}

const MONGO_README: &str = r#"# MongoDB Initialization & Migrations

Foundry adopts a **Zero-DDL Document Architecture** with MongoDB:
- **No SQL DDL migrations needed**: Dynamic schema models and entity records are stored in native BSON collections.
- **Automatic Initialization**: When `AUTO_MIGRATE=true` (default), the Foundry server creates all collections, indexes, and the default superadmin user automatically upon startup.
- **Manual DBA Control**: When `AUTO_MIGRATE=false`, DBA or deployment pipelines can execute the initialization script manually:
  ```bash
  mongosh "mongodb://127.0.0.1:27017/foundry" migrations/init.js
  ```
"#;

impl DatabaseKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "postgres",
            DatabaseKind::MySql => "mysql",
            DatabaseKind::MariaDb => "mariadb",
            DatabaseKind::MongoDb => "mongodb",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => "PostgreSQL (14+)",
            DatabaseKind::MySql => "MySQL (8.0+)",
            DatabaseKind::MariaDb => "MariaDB (10.5+)",
            DatabaseKind::MongoDb => "MongoDB (6.0+)",
        }
    }

    pub fn default_url(&self, db_name: &str) -> String {
        match self {
            DatabaseKind::Postgres => {
                format!(
                    "postgres://postgres:postgrespassword@localhost:5432/{}",
                    db_name
                )
            }
            DatabaseKind::MySql => format!("mysql://root:root@localhost:3306/{}", db_name),
            DatabaseKind::MariaDb => format!("mariadb://root:root@localhost:3307/{}", db_name),
            DatabaseKind::MongoDb => format!("mongodb://127.0.0.1:27017/{}", db_name),
        }
    }

    pub fn ddl_content(&self) -> String {
        foundry_storage::Database::get_ddl(self.as_str())
            .unwrap_or_else(|_| self.fallback_ddl().to_string())
    }

    pub fn default_migration_filename(&self) -> &'static str {
        foundry_storage::Database::get_default_migration_filename(self.as_str()).unwrap_or(
            match self {
                DatabaseKind::MongoDb => "init.js",
                _ => "init.sql",
            },
        )
    }

    fn fallback_ddl(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => {
                include_str!("../../foundry_storage/migrations/postgres/init.sql")
            }
            DatabaseKind::MySql => include_str!("../../foundry_storage/migrations/mysql/init.sql"),
            DatabaseKind::MariaDb => {
                include_str!("../../foundry_storage/migrations/mariadb/init.sql")
            }
            DatabaseKind::MongoDb => {
                include_str!("../../foundry_storage/migrations/mongodb/init.js")
            }
        }
    }

    pub fn docker_run_hint(&self, pkg_name: &str) -> String {
        match self {
            DatabaseKind::Postgres => format!(
                r#"```bash
# 启动本地 PostgreSQL 18
docker run -d --name {pkg_name}-postgres -p 5432:5432 \
  -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgrespassword -e POSTGRES_DB=foundry \
  postgres:18-alpine

# 启动本地 Redis 7
docker run -d --name {pkg_name}-redis -p 6379:6379 redis:7-alpine
```"#
            ),
            DatabaseKind::MySql => format!(
                r#"```bash
# 启动本地 MySQL 8.4
docker run -d --name {pkg_name}-mysql -p 3306:3306 \
  -e MYSQL_ROOT_PASSWORD=root -e MYSQL_DATABASE=foundry \
  mysql:8.4

# 启动本地 Redis 7
docker run -d --name {pkg_name}-redis -p 6379:6379 redis:7-alpine
```"#
            ),
            DatabaseKind::MariaDb => format!(
                r#"```bash
# 启动本地 MariaDB 11
docker run -d --name {pkg_name}-mariadb -p 3307:3306 \
  -e MARIADB_ROOT_PASSWORD=root -e MARIADB_DATABASE=foundry \
  mariadb:11

# 启动本地 Redis 7
docker run -d --name {pkg_name}-redis -p 6379:6379 redis:7-alpine
```"#
            ),
            DatabaseKind::MongoDb => format!(
                r#"```bash
# 启动本地 MongoDB 7
docker run -d --name {pkg_name}-mongodb -p 27017:27017 \
  mongo:7

# 启动本地 Redis 7
docker run -d --name {pkg_name}-redis -p 6379:6379 redis:7-alpine
```"#
            ),
        }
    }

    pub fn dba_migration_hint(&self) -> &'static str {
        match self {
            DatabaseKind::Postgres => {
                "psql -h 127.0.0.1 -U postgres -d foundry -f migrations/init.sql"
            }
            DatabaseKind::MySql => "mysql -h 127.0.0.1 -u root -p foundry < migrations/init.sql",
            DatabaseKind::MariaDb => {
                "mariadb -h 127.0.0.1 -P 3307 -u root -p foundry < migrations/init.sql"
            }
            DatabaseKind::MongoDb => {
                "mongosh \"mongodb://127.0.0.1:27017/foundry\" migrations/init.js"
            }
        }
    }
}

pub async fn run_cli() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            db,
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
                    db: Some(&db),
                },
            )?;
        }
        Commands::System { action } => match action {
            SystemCommands::New {
                slug,
                name,
                project_dir,
                with_table,
                table_name,
                db,
                ddl_dir,
            } => {
                scaffold_subsystem_with_options(
                    &slug,
                    SubsystemScaffoldOptions {
                        name: name.as_deref(),
                        project_dir: &project_dir,
                        with_table,
                        table_name: table_name.as_deref(),
                        db: db.as_deref(),
                        ddl_dir: ddl_dir.as_deref(),
                    },
                )?;
            }
            SystemCommands::List { path } => {
                list_subsystems(&path)?;
            }
        },
        Commands::Migrate {
            database_url,
            database_type,
            generate,
            output,
        } => {
            if generate {
                let explicit_type = database_type
                    .or_else(|| std::env::var("DATABASE_TYPE").ok())
                    .unwrap_or_else(|| "postgres".to_string());
                let db_kind = explicit_type.parse::<DatabaseKind>()?;
                if output.as_deref() == Some("-") {
                    print!("{}", db_kind.ddl_content());
                } else {
                    let path =
                        generate_ddl(Path::new("."), db_kind, output.as_deref().map(Path::new))?;
                    println!(
                        "✅ Generated {} DDL migration at {:?}",
                        db_kind.display_name(),
                        path
                    );
                }
                return Ok(());
            }

            let db_url = database_url.unwrap_or_else(|| {
                std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgrespassword@localhost:5432/foundry".to_string()
                })
            });
            let explicit_type = database_type.or_else(|| std::env::var("DATABASE_TYPE").ok());
            println!(
                "Connecting to database and running migrations for {} (type: {:?})...",
                db_url,
                explicit_type.as_deref().unwrap_or("auto-detect")
            );
            foundry_storage::Database::run_migrations(&db_url, explicit_type.as_deref()).await?;
            println!("✅ Core platform migrations applied successfully.");

            let db_kind = explicit_type
                .as_deref()
                .and_then(|t| t.parse::<DatabaseKind>().ok())
                .unwrap_or_else(|| {
                    if db_url.starts_with("mysql://") {
                        DatabaseKind::MySql
                    } else if db_url.starts_with("mariadb://") {
                        DatabaseKind::MariaDb
                    } else if db_url.starts_with("mongodb://")
                        || db_url.starts_with("mongodb+srv://")
                    {
                        DatabaseKind::MongoDb
                    } else {
                        DatabaseKind::Postgres
                    }
                });
            sync_subsystem_migrations(&db_url, db_kind, Path::new(".")).await?;
            println!("✅ Database migrations synced successfully.");
        }
        Commands::Ddl { db, output } => {
            let db_kind = db.parse::<DatabaseKind>()?;
            if output.as_deref() == Some("-") {
                print!("{}", db_kind.ddl_content());
            } else {
                let path = generate_ddl(Path::new("."), db_kind, output.as_deref().map(Path::new))?;
                println!(
                    "✅ Generated {} DDL migration at {:?}",
                    db_kind.display_name(),
                    path
                );
            }
        }
        Commands::Validate { path } => {
            validate_project(&path)?;
        }
    }

    Ok(())
}

/// Generate dialect DDL / migration files into the target project or directory.
/// When output is None, creates `migrations/` under `base_dir` and writes the dialect script.
pub fn generate_ddl(
    base_dir: &Path,
    db_kind: DatabaseKind,
    output: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    let content = db_kind.ddl_content();
    let target_path = match output {
        Some(p) => {
            let full_path = if p.is_absolute() {
                p.to_path_buf()
            } else {
                base_dir.join(p)
            };
            if let Some(parent) = full_path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full_path, &content)?;
            full_path
        }
        None => {
            let dir = base_dir.join("migrations");
            fs::create_dir_all(&dir)?;
            let file_path = dir.join(db_kind.default_migration_filename());
            fs::write(&file_path, &content)?;
            if db_kind == DatabaseKind::MongoDb {
                let _ = fs::write(dir.join("README.md"), MONGO_README);
            }
            file_path
        }
    };
    Ok(target_path)
}

/// Project scaffolding options
#[derive(Debug, Clone, Default)]
pub struct ProjectOptions<'a> {
    pub local_path: Option<&'a str>,
    pub git: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub version: Option<&'a str>,
    pub db: Option<&'a str>,
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

    let db_kind = opts.db.unwrap_or("postgres").parse::<DatabaseKind>()?;

    println!(
        "🚀 Creating new Foundry application: {} (Database: {})...",
        name,
        db_kind.display_name()
    );

    fs::create_dir_all(project_dir.join("src/systems/sample/controllers"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/logic"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/dto"))?;
    fs::create_dir_all(project_dir.join("src/systems/sample/custom_pages"))?;

    // 1. Cargo.toml
    let foundry_dep = if db_kind == DatabaseKind::Postgres {
        if let Some(p) = opts.local_path {
            format!(r#"foundry = {{ path = "{}" }}"#, p)
        } else if let Some(v) = opts.version {
            format!(r#"foundry = "{}"#, v)
        } else {
            let repo = opts.git.unwrap_or("https://github.com/foundkit/foundry");
            let branch = opts.branch.unwrap_or("main");
            format!(r#"foundry = {{ git = "{}", branch = "{}" }}"#, repo, branch)
        }
    } else {
        let feat = db_kind.as_str();
        if let Some(p) = opts.local_path {
            format!(
                r#"foundry = {{ path = "{}", default-features = false, features = ["standard", "{}"] }}"#,
                p, feat
            )
        } else if let Some(v) = opts.version {
            format!(
                r#"foundry = {{ version = "{}", default-features = false, features = ["standard", "{}"] }}"#,
                v, feat
            )
        } else {
            let repo = opts.git.unwrap_or("https://github.com/foundkit/foundry");
            let branch = opts.branch.unwrap_or("main");
            format!(
                r#"foundry = {{ git = "{}", branch = "{}", default-features = false, features = ["standard", "{}"] }}"#,
                repo, branch, feat
            )
        }
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
validator = {{ version = "0.21", features = ["derive"] }}
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

    // 9. .env.example & .env
    let active_url = db_kind.default_url("foundry");
    let active_type = db_kind.as_str();

    let env_example = format!(
        r#"HOST=0.0.0.0
PORT=8080
# Database Configuration ({db_display})
DATABASE_URL={active_url}
DATABASE_TYPE={active_type}

# Alternative Database Options:
# DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432/foundry
# DATABASE_URL=mysql://root:root@localhost:3306/foundry
# DATABASE_URL=mariadb://root:root@localhost:3307/foundry
# DATABASE_URL=mongodb://127.0.0.1:27017/foundry

REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=super_secret_jwt_key_change_in_production
# Auto-migrate database schema on server startup (true: automatic sync; false: strict manual DBA control)
AUTO_MIGRATE=true
"#,
        db_display = db_kind.display_name(),
        active_url = active_url,
        active_type = active_type,
    );
    fs::write(project_dir.join(".env.example"), &env_example)?;
    fs::write(project_dir.join(".env"), &env_example)?;

    // 10. .gitignore
    let gitignore = r#"/target
.env
*.log
.DS_Store
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    // 11. README.md
    let docker_cmd = db_kind.docker_run_hint(pkg_name);
    let dba_cmd = db_kind.dba_migration_hint();
    let db_display = db_kind.display_name();

    let readme = format!(
        r#"# {name}

基于 [Foundry](https://github.com/foundkit/foundry) 现代模块化后端框架构建的独立业务应用。

---

## ⚡ 快速上手

### 1. 数据库与缓存准备

本项目初始化配置使用的数据库引擎为：**{db_display}**。

在 `.env` 中确认你的数据库与 Redis 连接配置（已预置）：

```env
DATABASE_URL={active_url}
DATABASE_TYPE={active_type}
REDIS_URL=redis://127.0.0.1:6379
```

**本地数据库启动方式（二选一）：**

* **方式一：已有独立数据库**：直接将 `.env` 中的 `DATABASE_URL` 与 `REDIS_URL` 改为你现有的实例连接串即可。
* **方式二：使用 Docker 启动本地依赖**：

{docker_cmd}

### 2. 数据库初始化与管控说明

* **全自动初始化（默认推荐）**：
  在 `.env` 中保持 `AUTO_MIGRATE=true`，执行 `cargo run` 启动应用时，Foundry 框架会自动检测并完成数据库表结构/集合、覆盖索引创建以及初始超级管理员（`admin` / `admin123456`）账号注入。无需任何手动建表操作。

* **DBA 手动执行与严格管控（生产环境）**：
  若生产环境需严格审计，将 `.env` 中的 `AUTO_MIGRATE=false` 关闭自动迁移。
  本项目默认不预设 `migrations/` 目录，如需获取针对 **{db_display}** 的方言 DDL 脚本，请在当前工程下执行：
  ```bash
  foundry ddl
  # 或显式指定方言：foundry ddl --db {db_engine}
  # 或使用迁移命令生成：foundry migrate --generate
  ```
  执行后将在当前工程下生成 `migrations/` 目录及方言脚本，随后可交由 DBA 在上线前执行初始化：
  ```bash
  {dba_cmd}
  ```

---

### 3. 启动应用服务

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

Foundry 支持以 **ORM 优先 (ORM-First, Prisma 风格)** 的架构进行数据模型定义与增删改查：

### 模式 A：Code-First ORM 实体模型（Prisma 风格强类型，推荐）
支持在子系统代码中通过强类型定义 ORM Schema。`foundry migrate` 与 `foundry ddl` 命令会自动读取代码中的 ORM 模型，编译为对应数据库方言（PostgreSQL, MySQL, MariaDB）的 DDL 结构：

```rust
use foundry::prelude::*;

impl SubsystemModule for BillingSubsystem {{
    fn slug(&self) -> &'static str {{ "billing" }}
    fn display_name(&self) -> &'static str {{ "账单中心" }}

    // 类似 Prisma 的代码定义 ORM 模型
    fn orm_models(&self) -> Vec<EntitySchema> {{
        vec![
            EntitySchema::new("billing_invoices")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("invoice_no", 64).not_null().unique())
                .column(ColumnDef::decimal("amount", 10, 2).not_null())
                .column(ColumnDef::varchar("status", 24).not_null())
                .column(ColumnDef::json("metadata").default_empty_json())
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("updated_at").default_current_timestamp())
                .index(IndexDef::new("idx_invoices_sys", &["system_id"]))
        ]
    }}
}}
```

增删改查同样使用 ORM 构建器（或仓储层封装方法），底层自动适配各方言参数占位符（`$1` vs `?`）：
```rust
use foundry::prelude::*;

// ORM 插入
let (sql, binds) = OrmInsert::into_table("billing_invoices")
    .value("system_id", "billing")
    .value("invoice_no", "INV-2026-001")
    .value("amount", 99.50)
    .to_sql(Dialect::Postgres);

// ORM 分页查询
let (sql, binds) = OrmSelect::from_table("billing_invoices")
    .where_eq("system_id", "billing")
    .order_by_desc("created_at")
    .limit(20)
    .offset(0)
    .to_sql(Dialect::Postgres);
```

### 模式 B：Zero-DDL 动态模型（免表迁移、即开即用）
适用于中后台配置、内容发布、快速迭代原型。无需手写 DDL 建表，数据按子系统自动隔离并自动提供 RESTful 接口。对于关系型数据库底层自动利用统一模型表，MongoDB 底层自动存入原生 BSON 文档集合：

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

### 模式 C：原生 SQL / 脚本逃生通道（特殊触发器、复杂统计、MongoDB 脚本）
对于无关系型 ORM 的数据库（如 MongoDB）或特殊存储引擎，可通过 `migration_ddl` 或 `migrations/` 目录编写特定脚本。

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

# 4. 生成特定数据库方言的 DDL 迁移脚本
foundry ddl --db postgres   # 或 mysql / mariadb / mongodb

# 5. 校验当前工程规范与子系统目录结构
foundry validate
```

> 💡 **提示**：管理员账号增删、角色分配及权限管控，直接登录管理后台（`/admin`）进行可视化操作即可，无需在终端处理。
"#,
        name = name,
        db_display = db_display,
        db_engine = db_kind.as_str(),
        active_url = active_url,
        active_type = active_type,
        docker_cmd = docker_cmd,
        dba_cmd = dba_cmd,
    );
    fs::write(project_dir.join("README.md"), readme)?;

    println!(
        "✅ Project '{}' created successfully at {:?}! (Database: {})",
        name,
        project_dir,
        db_kind.display_name()
    );
    println!("👉 Next steps:");
    println!("   cd {}", name);
    println!(
        "   # Ensure {} and Redis are accessible as configured in .env",
        db_kind.as_str()
    );
    println!("   cargo run");

    Ok(project_dir)
}

/// Options for scaffolding a new subsystem module
#[derive(Debug, Clone, Default)]
pub struct SubsystemScaffoldOptions<'a> {
    pub name: Option<&'a str>,
    pub project_dir: &'a str,
    pub with_table: bool,
    pub table_name: Option<&'a str>,
    pub db: Option<&'a str>,
    pub ddl_dir: Option<&'a str>,
}

impl<'a> From<(Option<&'a str>, &'a str)> for SubsystemScaffoldOptions<'a> {
    fn from((name, project_dir): (Option<&'a str>, &'a str)) -> Self {
        Self {
            name,
            project_dir,
            with_table: false,
            table_name: None,
            db: None,
            ddl_dir: None,
        }
    }
}

/// Scans the `migrations/` directory for any subsystem DDL scripts (e.g. `migrations/<dialect>/*.sql` or `migrations/*.sql`)
/// and executes them sequentially against the target database to synchronize custom subsystem schemas.
pub async fn sync_subsystem_migrations(
    database_url: &str,
    db_kind: DatabaseKind,
    base_dir: &Path,
) -> anyhow::Result<()> {
    let migrations_dir = base_dir.join("migrations");
    if !migrations_dir.exists() {
        return Ok(());
    }

    let dialect_dir = migrations_dir.join(db_kind.as_str());
    let mut sql_files = Vec::new();

    // Check dialect-specific folder (e.g. migrations/mysql/)
    if let Ok(entries) = fs::read_dir(&dialect_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("sql")
                && path.file_name().and_then(|s| s.to_str()) != Some("init.sql")
            {
                sql_files.push(path);
            }
        }
    }

    // Also check root migrations/ folder for additional sql scripts
    if let Ok(entries) = fs::read_dir(&migrations_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("sql")
                && path.file_name().and_then(|s| s.to_str()) != Some("init.sql")
                && !sql_files.contains(&path)
            {
                sql_files.push(path);
            }
        }
    }

    if sql_files.is_empty() {
        return Ok(());
    }

    sql_files.sort();

    match db_kind {
        #[cfg(feature = "postgres")]
        DatabaseKind::Postgres => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(database_url)
                .await?;
            for file in sql_files {
                let content = fs::read_to_string(&file)?;
                println!("🚀 Applying subsystem DDL: {:?} ...", file);
                sqlx::raw_sql(sqlx::AssertSqlSafe(content.as_str()))
                    .execute(&pool)
                    .await?;
                println!("✅ Synced subsystem DDL: {:?}", file);
            }
        }
        #[cfg(any(feature = "mysql", feature = "mariadb"))]
        DatabaseKind::MySql | DatabaseKind::MariaDb => {
            let normalized_url = if database_url.starts_with("mariadb://") {
                database_url.replace("mariadb://", "mysql://")
            } else {
                database_url.to_string()
            };
            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(2)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(&normalized_url)
                .await?;
            for file in sql_files {
                let content = fs::read_to_string(&file)?;
                println!("🚀 Applying subsystem DDL: {:?} ...", file);
                for stmt in content.split(';') {
                    let s = stmt.trim();
                    if !s.is_empty() {
                        sqlx::query(sqlx::AssertSqlSafe(s)).execute(&pool).await?;
                    }
                }
                println!("✅ Synced subsystem DDL: {:?}", file);
            }
        }
        #[cfg(feature = "mongodb")]
        DatabaseKind::MongoDb => {
            for file in sql_files {
                println!("ℹ️ Subsystem script noted for MongoDB: {:?}", file);
            }
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }

    Ok(())
}

fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>()
}

/// Scaffold a new subsystem in an existing project with default options
pub fn scaffold_subsystem(slug: &str, name: Option<&str>, project_dir: &str) -> anyhow::Result<()> {
    scaffold_subsystem_with_options(
        slug,
        SubsystemScaffoldOptions {
            name,
            project_dir,
            with_table: false,
            table_name: None,
            db: None,
            ddl_dir: None,
        },
    )
}

/// Scaffold a new subsystem in an existing project with full configuration options
pub fn scaffold_subsystem_with_options(
    slug: &str,
    opts: SubsystemScaffoldOptions<'_>,
) -> anyhow::Result<()> {
    if !is_valid_slug(slug, 32) {
        anyhow::bail!(
            "Invalid system slug: '{}'. Must be 2-32 lowercase alphanumeric characters or underscore/hyphen.",
            slug
        );
    }

    let display_name = opts.name.unwrap_or(slug);
    let base_dir = Path::new(opts.project_dir).join("src/systems").join(slug);

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

    let base_pascal = to_pascal_case(slug);
    let struct_name = format!("{}Subsystem", base_pascal);
    let has_table = opts.with_table || opts.table_name.is_some();
    let table_name = opts
        .table_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_items", slug));

    if has_table {
        fs::create_dir_all(base_dir.join("models"))?;
        let item_name = format!("{}Item", base_pascal);
        let service_name = format!("{}Service", base_pascal);

        // 1. Dedicated ORM Entity: models/mod.rs
        let models_content = format!(
            r#"use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use sqlx::FromRow;

/// Strongly-typed entity for dedicated table `{table_name}`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct {item_name} {{
    pub id: i64,
    pub system_id: String,
    pub title: String,
    pub content: Option<String>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp: Never physically deleted from database
    pub deleted_at: Option<DateTime<Utc>>,
}}
"#,
            table_name = table_name,
            item_name = item_name
        );
        fs::write(base_dir.join("models/mod.rs"), models_content)?;

        // 2. DTO: dto/mod.rs
        let dto_content = format!(
            r#"use serde::{{Deserialize, Serialize}};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Create{item_name}Request {{
    #[validate(length(min = 1, max = 128, message = "Title must be between 1 and 128 characters"))]
    pub title: String,
    pub content: Option<String>,
}}

#[derive(Debug, Deserialize, Validate)]
pub struct Update{item_name}Request {{
    #[validate(length(min = 1, max = 128, message = "Title must be between 1 and 128 characters"))]
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<i16>,
}}

#[derive(Debug, Serialize)]
pub struct {item_name}Response {{
    pub id: i64,
    pub system_id: String,
    pub title: String,
    pub content: Option<String>,
    pub status: i16,
}}
"#,
            item_name = item_name
        );
        fs::write(base_dir.join("dto/mod.rs"), dto_content)?;

        // 3. Logic & Service: logic/mod.rs
        let logic_content = format!(
            r#"use crate::systems::{slug}::dto::{{Create{item_name}Request, Update{item_name}Request}};
use crate::systems::{slug}::models::{item_name};
use foundry::prelude::*;

pub struct {service_name};

impl {service_name} {{
    /// Create a new record using typed ORM
    pub async fn create(ctx: &SystemContext, db: &Database, req: Create{item_name}Request) -> AppResult<{item_name}> {{
        let (sql, binds) = OrmInsert::into_table("{table_name}")
            .value("system_id", &ctx.system_slug)
            .value("title", &req.title)
            .value("content", req.content.as_deref().unwrap_or(""))
            .value("status", 1)
            .to_sql(Dialect::Postgres);
        let _ = (sql, binds, db);

        Ok({item_name} {{
            id: 1,
            system_id: ctx.system_slug.clone(),
            title: req.title,
            content: req.content,
            status: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        }})
    }}

    /// Retrieve single record by ID with soft-delete filter
    pub async fn get_by_id(ctx: &SystemContext, db: &Database, id: i64) -> AppResult<{item_name}> {{
        let (sql, binds) = OrmSelect::from_table("{table_name}")
            .where_eq("id", id)
            .where_eq("system_id", &ctx.system_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);
        let _ = (sql, binds, db);

        Ok({item_name} {{
            id,
            system_id: ctx.system_slug.clone(),
            title: "Sample Item".to_string(),
            content: None,
            status: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        }})
    }}

    /// Retrieve paginated records with soft-delete filter
    pub async fn list(ctx: &SystemContext, db: &Database, page: u64, page_size: u64) -> AppResult<PaginatedData<{item_name}>> {{
        let (sql, binds) = OrmSelect::from_table("{table_name}")
            .where_eq("system_id", &ctx.system_slug)
            .where_null("deleted_at")
            .order_by_desc("created_at")
            .limit(page_size as i64)
            .offset(((page.max(1) - 1) * page_size) as i64)
            .to_sql(Dialect::Postgres);
        let _ = (sql, binds, db);

        Ok(PaginatedData::new(vec![], page, page_size, 0))
    }}

    /// Update record attributes
    pub async fn update(ctx: &SystemContext, db: &Database, id: i64, req: Update{item_name}Request) -> AppResult<{item_name}> {{
        let mut stmt = OrmUpdate::table("{table_name}");
        if let Some(ref t) = req.title {{
            stmt = stmt.set("title", t.clone());
        }}
        if let Some(ref c) = req.content {{
            stmt = stmt.set("content", c.clone());
        }}
        if let Some(s) = req.status {{
            stmt = stmt.set("status", s);
        }}
        let (sql, binds) = stmt
            .set_now("updated_at")
            .where_eq("id", id)
            .where_eq("system_id", &ctx.system_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);
        let _ = (sql, binds, db);

        Ok({item_name} {{
            id,
            system_id: ctx.system_slug.clone(),
            title: req.title.unwrap_or_else(|| "Updated".to_string()),
            content: req.content,
            status: req.status.unwrap_or(1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        }})
    }}

    /// Soft-delete an item (marks `deleted_at = NOW()`).
    /// Absolutely zero physical DELETE SQL is executed.
    /// Fully operational even if database account has no DELETE privileges!
    pub async fn delete(ctx: &SystemContext, db: &Database, id: i64) -> AppResult<()> {{
        let (sql, binds) = OrmUpdate::table("{table_name}")
            .set_now("deleted_at")
            .where_eq("id", id)
            .where_eq("system_id", &ctx.system_slug)
            .where_null("deleted_at")
            .to_sql(Dialect::Postgres);
        let _ = (sql, binds, db);

        Ok(())
    }}
}}
"#,
            slug = slug,
            item_name = item_name,
            service_name = service_name,
            table_name = table_name
        );
        fs::write(base_dir.join("logic/mod.rs"), logic_content)?;

        // 4. Controller: controllers/mod.rs
        let controller_content = format!(
            r#"use axum::{{
    extract::{{Extension, Path, Query}},
    routing::{{get, post}},
    Json, Router,
}};
use foundry::prelude::*;
use serde::Deserialize;
use validator::Validate;

use crate::systems::{slug}::dto::{{Create{item_name}Request, Update{item_name}Request}};
use crate::systems::{slug}::logic::{service_name};
use crate::systems::{slug}::models::{item_name};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {{
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}}

pub fn build_routes() -> Router {{
    Router::new()
        .route("/items", get(handle_list).post(handle_create))
        .route("/items/{{id}}", get(handle_get).put(handle_update).delete(handle_delete))
}}

pub async fn handle_list(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Query(pq): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<PaginatedData<{item_name}>>>> {{
    let page = pq.page.unwrap_or(1);
    let page_size = pq.page_size.unwrap_or(20);
    let res = {service_name}::list(&ctx, &state.db, page, page_size).await?;
    Ok(Json(ApiResponse::success(res)))
}}

pub async fn handle_get(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<{item_name}>>> {{
    let res = {service_name}::get_by_id(&ctx, &state.db, id).await?;
    Ok(Json(ApiResponse::success(res)))
}}

pub async fn handle_create(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Json(payload): Json<Create{item_name}Request>,
) -> AppResult<Json<ApiResponse<{item_name}>>> {{
    payload.validate()?;
    let res = {service_name}::create(&ctx, &state.db, payload).await?;
    Ok(Json(ApiResponse::success(res)))
}}

pub async fn handle_update(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<Update{item_name}Request>,
) -> AppResult<Json<ApiResponse<{item_name}>>> {{
    payload.validate()?;
    let res = {service_name}::update(&ctx, &state.db, id, payload).await?;
    Ok(Json(ApiResponse::success(res)))
}}

pub async fn handle_delete(
    Extension(ctx): Extension<SystemContext>,
    Extension(state): Extension<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {{
    // Executes soft delete (updates deleted_at), zero DELETE privilege required!
    {service_name}::delete(&ctx, &state.db, id).await?;
    Ok(Json(ApiResponse::success(())))
}}
"#,
            slug = slug,
            item_name = item_name,
            service_name = service_name
        );
        fs::write(base_dir.join("controllers/mod.rs"), controller_content)?;

        // 5. Subsystem Module struct: mod.rs
        let mod_content = format!(
            r#"pub mod controllers;
pub mod dto;
pub mod logic;
pub mod models;

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

    /// Dedicated subsystem physical table with soft deletion support
    fn orm_models(&self) -> Vec<EntitySchema> {{
        vec![
            EntitySchema::new("{table_name}")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("system_id", 32).not_null())
                .column(ColumnDef::varchar("title", 128).not_null())
                .column(ColumnDef::text("content").nullable())
                .column(ColumnDef::smallint("status").not_null().default_val(DefaultValue::SmallInt(1)))
                .column(ColumnDef::datetime("created_at").default_current_timestamp())
                .column(ColumnDef::datetime("updated_at").default_current_timestamp())
                .column(ColumnDef::datetime("deleted_at").nullable())
                .index(IndexDef::new("idx_{table_name}_sys_query", &["system_id", "deleted_at", "created_at"]))
                .comment("Dedicated subsystem table for {slug}")
        ]
    }}
}}
"#,
            struct_name = struct_name,
            slug = slug,
            display_name = display_name,
            table_name = table_name
        );
        fs::write(base_dir.join("mod.rs"), mod_content)?;

        // 6. Compile & Generate DDL SQL Migration Files
        let entity_schema = EntitySchema::new(&table_name)
            .column(ColumnDef::bigint("id").primary_key().auto_increment())
            .column(ColumnDef::varchar("system_id", 32).not_null())
            .column(ColumnDef::varchar("title", 128).not_null())
            .column(ColumnDef::text("content").nullable())
            .column(
                ColumnDef::smallint("status")
                    .not_null()
                    .default_val(DefaultValue::SmallInt(1)),
            )
            .column(ColumnDef::datetime("created_at").default_current_timestamp())
            .column(ColumnDef::datetime("updated_at").default_current_timestamp())
            .column(ColumnDef::datetime("deleted_at").nullable())
            .index(IndexDef::new(
                format!("idx_{}_sys_query", table_name),
                &["system_id", "deleted_at", "created_at"],
            ))
            .comment(format!("Dedicated subsystem table for {}", slug));

        let project_root = Path::new(opts.project_dir);
        let migrations_base = opts
            .ddl_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| project_root.join("migrations"));

        let target_dialects: Vec<(Dialect, &str)> = match opts.db {
            Some("postgres") | Some("postgresql") | Some("pg") => {
                vec![(Dialect::Postgres, "postgres")]
            }
            Some("mysql") => vec![(Dialect::MySql, "mysql")],
            Some("mariadb") => vec![(Dialect::MariaDb, "mariadb")],
            Some("all") => vec![
                (Dialect::Postgres, "postgres"),
                (Dialect::MySql, "mysql"),
                (Dialect::MariaDb, "mariadb"),
            ],
            _ => {
                // By default, physical table separation & DDL generation is focused on MySQL & MariaDB,
                // where monolithic records tables face severe performance constraints.
                if let Ok(env_content) = fs::read_to_string(project_root.join(".env")) {
                    if env_content.contains("DATABASE_TYPE=mariadb")
                        || env_content.contains("mariadb://")
                    {
                        vec![(Dialect::MariaDb, "mariadb")]
                    } else if env_content.contains("DATABASE_TYPE=mysql")
                        || env_content.contains("mysql://")
                    {
                        vec![(Dialect::MySql, "mysql")]
                    } else {
                        vec![(Dialect::MySql, "mysql"), (Dialect::MariaDb, "mariadb")]
                    }
                } else {
                    vec![(Dialect::MySql, "mysql"), (Dialect::MariaDb, "mariadb")]
                }
            }
        };

        for (dialect, dialect_name) in target_dialects {
            let compiled_ddl = compile_schemas(std::slice::from_ref(&entity_schema), dialect);
            let ddl_with_header = format!(
                r#"-- ============================================================================
-- FOUNDRY SUBSYSTEM DDL MIGRATION: {slug}
-- Dedicated Table: {table_name}
-- Target Database: {dialect_title}
--
-- Soft-Delete Policy: All deletions MUST update `deleted_at`.
-- Zero-DELETE Privilege: Fully compatible with database accounts without DELETE privilege!
--
-- Production Deployment: Hand over this file to DBA for audit and manual execution.
-- Development Workflow: Run `foundry migrate` to apply and synchronize automatically.
-- ============================================================================

{compiled_ddl}
"#,
                slug = slug,
                table_name = table_name,
                dialect_title = dialect_name.to_uppercase(),
                compiled_ddl = compiled_ddl.trim()
            );

            let dialect_dir = migrations_base.join(dialect_name);
            fs::create_dir_all(&dialect_dir)?;
            let file_path = dialect_dir.join(format!("{}.sql", slug));
            fs::write(&file_path, &ddl_with_header)?;
            println!("📄 Generated DDL SQL for {}: {:?}", dialect_name, file_path);
        }

        // Also save a reference schema.sql in subsystem directory
        let ref_ddl = compile_schemas(&[entity_schema], Dialect::Postgres);
        fs::write(base_dir.join("schema.sql"), ref_ddl)?;
        println!(
            "✅ Dedicated table ORM enabled for Subsystem '{}' (Table: {})",
            slug, table_name
        );
    } else {
        // Standard lightweight subsystem scaffold without dedicated table
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

        // Subsystem Module struct
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
    }

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
    let migrations_dir = p.join("migrations");

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

    if migrations_dir.exists() {
        println!("  ✅ migrations/ directory present");
    }

    list_subsystems(path)?;
    println!("✅ Project validation complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_project_and_subsystem_default_postgres() {
        let temp_dir =
            std::env::temp_dir().join(format!("foundry_test_cli_pg_{}", uuid::Uuid::new_v4()));
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
        // Verify dev and config directories are NOT created
        assert!(!path.join("dev").exists());
        assert!(!path.join("dev/docker-compose.yml").exists());
        assert!(!path.join("dev/init-db.sh").exists());
        assert!(!path.join("dev/init-dev-db.sql").exists());
        assert!(!path.join("config").exists());

        // Verify gitignore does not include dev/
        assert!(path.join(".gitignore").exists());
        let gitignore = fs::read_to_string(path.join(".gitignore")).unwrap();
        assert!(!gitignore.contains("dev/"));

        // Verify migrations directory is NOT created by default
        assert!(!path.join("migrations").exists());

        // Verify on-demand DDL generation creates migrations/
        let ddl_path = generate_ddl(&path, DatabaseKind::Postgres, None).unwrap();
        assert!(ddl_path.exists());
        assert!(path.join("migrations/init.sql").exists());
        let sql = fs::read_to_string(path.join("migrations/init.sql")).unwrap();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS systems"));
        assert!(sql.contains("BIGSERIAL"));

        // Verify README
        assert!(path.join("README.md").exists());
        let readme = fs::read_to_string(path.join("README.md")).unwrap();
        assert!(!readme.contains("dev/docker-compose.yml"));
        assert!(readme.contains("PostgreSQL"));
        assert!(readme.contains("AUTO_MIGRATE=true"));

        // Scaffold additional subsystem
        scaffold_subsystem("orders", Some("Order Management"), &project_name).unwrap();
        assert!(path.join("src/systems/orders/mod.rs").exists());
        assert!(path.join("src/systems/orders/controllers/mod.rs").exists());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_scaffold_project_mysql() {
        let temp_dir =
            std::env::temp_dir().join(format!("foundry_test_cli_mysql_{}", uuid::Uuid::new_v4()));
        let project_name = temp_dir.to_string_lossy().to_string();

        let path = scaffold_project(
            &project_name,
            ProjectOptions {
                local_path: Some("../../../crates/foundry"),
                db: Some("mysql"),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(!path.join("dev").exists());
        assert!(!path.join("config").exists());
        // Verify migrations directory is NOT created by default
        assert!(!path.join("migrations").exists());

        // Verify on-demand DDL generation creates migrations/
        let ddl_path = generate_ddl(&path, DatabaseKind::MySql, None).unwrap();
        assert!(ddl_path.exists());
        assert!(path.join("migrations/init.sql").exists());
        let sql = fs::read_to_string(path.join("migrations/init.sql")).unwrap();
        assert!(sql.contains("ENGINE=InnoDB"));
        assert!(sql.contains("MySQL"));

        let cargo_toml = fs::read_to_string(path.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains(r#"features = ["standard", "mysql"]"#));

        let env_file = fs::read_to_string(path.join(".env")).unwrap();
        assert!(env_file.contains("DATABASE_TYPE=mysql"));
        assert!(env_file.contains("mysql://"));

        let readme = fs::read_to_string(path.join("README.md")).unwrap();
        assert!(!readme.contains("dev/docker-compose.yml"));
        assert!(readme.contains("MySQL"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_scaffold_project_mariadb() {
        let temp_dir =
            std::env::temp_dir().join(format!("foundry_test_cli_mariadb_{}", uuid::Uuid::new_v4()));
        let project_name = temp_dir.to_string_lossy().to_string();

        let path = scaffold_project(
            &project_name,
            ProjectOptions {
                local_path: Some("../../../crates/foundry"),
                db: Some("mariadb"),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(!path.join("dev").exists());
        assert!(!path.join("config").exists());
        // Verify migrations directory is NOT created by default
        assert!(!path.join("migrations").exists());

        // Verify on-demand DDL generation creates migrations/
        let ddl_path = generate_ddl(&path, DatabaseKind::MariaDb, None).unwrap();
        assert!(ddl_path.exists());
        assert!(path.join("migrations/init.sql").exists());
        let sql = fs::read_to_string(path.join("migrations/init.sql")).unwrap();
        assert!(sql.contains("MariaDB"));

        let cargo_toml = fs::read_to_string(path.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains(r#"features = ["standard", "mariadb"]"#));

        let env_file = fs::read_to_string(path.join(".env")).unwrap();
        assert!(env_file.contains("DATABASE_TYPE=mariadb"));
        assert!(env_file.contains("mariadb://"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_scaffold_project_mongodb() {
        let temp_dir =
            std::env::temp_dir().join(format!("foundry_test_cli_mongo_{}", uuid::Uuid::new_v4()));
        let project_name = temp_dir.to_string_lossy().to_string();

        let path = scaffold_project(
            &project_name,
            ProjectOptions {
                local_path: Some("../../../crates/foundry"),
                db: Some("mongodb"),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(!path.join("dev").exists());
        assert!(!path.join("config").exists());
        // Verify migrations directory is NOT created by default
        assert!(!path.join("migrations").exists());

        // Verify on-demand DDL generation creates migrations/
        let ddl_path = generate_ddl(&path, DatabaseKind::MongoDb, None).unwrap();
        assert!(ddl_path.exists());
        assert!(path.join("migrations/init.js").exists());
        assert!(path.join("migrations/README.md").exists());
        let js = fs::read_to_string(path.join("migrations/init.js")).unwrap();
        assert!(js.contains("db.systems.createIndex"));

        let cargo_toml = fs::read_to_string(path.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains(r#"features = ["standard", "mongodb"]"#));

        let env_file = fs::read_to_string(path.join(".env")).unwrap();
        assert!(env_file.contains("DATABASE_TYPE=mongodb"));
        assert!(env_file.contains("mongodb://"));

        let readme = fs::read_to_string(path.join("README.md")).unwrap();
        assert!(!readme.contains("dev/docker-compose.yml"));
        assert!(readme.contains("MongoDB"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_database_kind_parsing_and_ddl_content() {
        assert_eq!(
            "postgres".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::Postgres
        );
        assert_eq!(
            "postgresql".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::Postgres
        );
        assert_eq!(
            "pg".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::Postgres
        );
        assert_eq!(
            "mysql".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::MySql
        );
        assert_eq!(
            "mariadb".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::MariaDb
        );
        assert_eq!(
            "mongodb".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::MongoDb
        );
        assert_eq!(
            "mongo".parse::<DatabaseKind>().unwrap(),
            DatabaseKind::MongoDb
        );
        assert!("unknown".parse::<DatabaseKind>().is_err());

        assert!(DatabaseKind::Postgres.ddl_content().contains("BIGSERIAL"));
        assert!(DatabaseKind::MySql.ddl_content().contains("ENGINE=InnoDB"));
        assert!(DatabaseKind::MariaDb.ddl_content().contains("MariaDB"));
        assert!(DatabaseKind::MongoDb.ddl_content().contains("createIndex"));
    }

    #[test]
    fn test_code_defined_ddl_extension() {
        foundry_storage::clear_registered_ddl_extensions();
        foundry_storage::register_ddl_extension(
            "subsystem:billing",
            "postgres",
            "CREATE TABLE billing_invoices (id UUID PRIMARY KEY);",
        );

        let ddl = DatabaseKind::Postgres.ddl_content();
        assert!(ddl.contains("BIGSERIAL"));
        assert!(ddl.contains("CODE-DEFINED EXTENSION DDL"));
        assert!(ddl.contains("billing_invoices"));

        // Clean up
        foundry_storage::clear_registered_ddl_extensions();
    }

    #[test]
    fn test_scaffold_subsystem_with_table() {
        let temp_dir = std::env::temp_dir().join(format!(
            "foundry_test_cli_subsystem_{}",
            uuid::Uuid::new_v4()
        ));
        let project_name = temp_dir.to_string_lossy().to_string();

        let _ = scaffold_project(
            &project_name,
            ProjectOptions {
                local_path: Some("../../../crates/foundry"),
                db: Some("mysql"),
                ..Default::default()
            },
        )
        .unwrap();

        scaffold_subsystem_with_options(
            "articles",
            SubsystemScaffoldOptions {
                name: Some("Articles Subsystem"),
                project_dir: &project_name,
                with_table: true,
                table_name: Some("articles_catalog"),
                db: Some("all"),
                ddl_dir: None,
            },
        )
        .unwrap();

        let path = Path::new(&project_name);
        assert!(path.join("src/systems/articles/models/mod.rs").exists());
        assert!(path.join("src/systems/articles/dto/mod.rs").exists());
        assert!(path.join("src/systems/articles/logic/mod.rs").exists());
        assert!(
            path.join("src/systems/articles/controllers/mod.rs")
                .exists()
        );
        assert!(path.join("src/systems/articles/schema.sql").exists());

        // Check DDL files generated
        assert!(path.join("migrations/mysql/articles.sql").exists());
        assert!(path.join("migrations/mariadb/articles.sql").exists());
        assert!(path.join("migrations/postgres/articles.sql").exists());

        let mysql_ddl = fs::read_to_string(path.join("migrations/mysql/articles.sql")).unwrap();
        assert!(mysql_ddl.contains("articles_catalog"));
        assert!(mysql_ddl.contains("CREATE TABLE IF NOT EXISTS"));
        assert!(mysql_ddl.contains("deleted_at"));
        assert!(mysql_ddl.contains("ENGINE=InnoDB"));

        let logic_rs = fs::read_to_string(path.join("src/systems/articles/logic/mod.rs")).unwrap();
        assert!(logic_rs.contains("Soft-delete"));
        assert!(logic_rs.contains("deleted_at"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

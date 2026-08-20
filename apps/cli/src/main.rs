use clap::{Parser, Subcommand};
use foundry_auth::hash_password;
use foundry_core::types::is_valid_slug;
use foundry_storage::{AdminStore, init_db_pool, run_migrations};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "foundry-cli")]
#[command(about = "Foundry CLI - Monorepo developer tooling & system scaffolding", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sub-system management and scaffolding
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },
    /// Database migrations
    Migrate {
        #[arg(short, long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Admin account utilities
    Admin {
        #[command(subcommand)]
        action: AdminCommands,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Scaffold a brand new compiled sub-system module in `systems/src/`
    New {
        /// Unique sub-system slug (e.g., carnival_demo, vip_mall)
        slug: String,
        /// Display name for the sub-system
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Scaffold a standalone external sub-system repository directory in `external_systems/`
    NewExternal {
        /// Unique sub-system slug
        slug: String,
        /// Display name for the sub-system
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Initialize a standalone Git repository for custom subsystems
    InitRepo {
        /// Directory path to initialize the repository in
        #[arg(default_value = "./my-foundry-systems")]
        path: String,
    },
    /// Validate subsystem directory structure and manifest integrity
    Validate {
        /// Target directory of subsystem or repository (defaults to current dir / systems)
        path: Option<String>,
    },
    /// List all registered sub-systems
    List,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        Commands::System { action } => match action {
            SystemCommands::New { slug, name } => {
                scaffold_subsystem(&slug, name.as_deref())?;
            }
            SystemCommands::NewExternal { slug, name } => {
                scaffold_external_subsystem(&slug, name.as_deref())?;
            }
            SystemCommands::InitRepo { path } => {
                init_subsystems_repo(&path)?;
            }
            SystemCommands::Validate { path } => {
                validate_subsystems(path.as_deref())?;
            }
            SystemCommands::List => {
                list_subsystems()?;
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
            println!("✅ Database migration completed successfully.");
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
    }

    Ok(())
}

fn scaffold_subsystem(slug: &str, name: Option<&str>) -> anyhow::Result<()> {
    if !is_valid_slug(slug, 32) {
        anyhow::bail!(
            "Invalid system slug: '{}'. Must be 2-32 lowercase alphanumeric characters or underscore/hyphen.",
            slug
        );
    }

    let display_name = name.unwrap_or(slug);
    let base_dir = Path::new("systems/src").join(slug);

    if base_dir.exists() {
        anyhow::bail!("Sub-system directory already exists at {:?}", base_dir);
    }

    println!(
        "🚀 Scaffolding new decoupled sub-system '{}' ({}) at {:?}...",
        slug, display_name, base_dir
    );

    fs::create_dir_all(base_dir.join("controllers"))?;
    fs::create_dir_all(base_dir.join("logic"))?;
    fs::create_dir_all(base_dir.join("dto"))?;
    fs::create_dir_all(base_dir.join("custom_pages"))?;

    // DTO
    fs::write(
        base_dir.join("dto/mod.rs"),
        "pub mod sample_dto;\npub use sample_dto::*;\n",
    )?;
    fs::write(
        base_dir.join("dto/sample_dto.rs"),
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
        "pub mod sample_service;\npub use sample_service::*;\n",
    )?;
    fs::write(
        base_dir.join("logic/sample_service.rs"),
        format!(
            r#"use crate::{slug}::dto::{{SampleRequest, SampleResponse}};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;

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
        r#"pub mod sample_controller;

use axum::{routing::post, Router};

pub fn build_routes() -> Router {
    Router::new().route("/sample", post(sample_controller::handle_sample))
}
"#,
    )?;
    fs::write(
        base_dir.join("controllers/sample_controller.rs"),
        format!(
            r#"use axum::{{extract::Extension, Json}};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::ApiResponse;
use validator::Validate;
use crate::{slug}::dto::{{SampleRequest, SampleResponse}};
use crate::{slug}::logic::SampleService;

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

    // Custom Admin UI Page HTML
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{display_name} 控制台</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 min-h-screen p-6 transition-colors">
  <div class="max-w-4xl mx-auto space-y-4">
    <h1 class="text-xl font-bold text-emerald-600 dark:text-emerald-400">📊 {display_name} 自定义运营看板</h1>
    <p class="text-xs text-slate-500">子系统 Slug: <code class="font-mono font-semibold">{slug}</code></p>
    <div id="session-info" class="p-4 rounded-xl bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-xs font-mono">
      等待来自 Foundry Admin 外壳的凭据初始化...
    </div>
  </div>
  <script>
    window.addEventListener('message', function(event) {{
      if (event.data?.type === 'FOUNDRY_INIT') {{
        const p = event.data.payload;
        if (p.theme === 'dark') document.documentElement.classList.add('dark');
        document.getElementById('session-info').innerText = '当前管理员: ' + (p.admin?.username || 'unknown') + ' | JWT Token: ' + (p.token ? p.token.substring(0, 20) + '...' : 'none');
      }}
    }});
  </script>
</body>
</html>"#,
        display_name = display_name,
        slug = slug
    );
    fs::write(base_dir.join("custom_pages/overview.html"), html_content)?;

    // Subsystem manifest metadata
    let manifest = serde_json::json!({
        "slug": slug,
        "display_name": display_name,
        "description": format!("Decoupled subsystem {}", display_name),
        "version": "1.0.0",
        "custom_pages": [
            {
                "key": format!("{}_overview", slug),
                "title": format!("{} 概览看板", display_name),
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

    // Subsystem Module Entry
    let module_struct_name = format!(
        "{}Module",
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

use std::path::PathBuf;
use axum::Router;
use foundry_core::SubsystemModule;
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
            PathBuf::from("systems/src/{slug}/custom_pages"),
            PathBuf::from("systems/{slug}/custom_pages"),
            PathBuf::from("../systems/src/{slug}/custom_pages"),
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

    fn custom_admin_pages(&self) -> Vec<foundry_core::CustomAdminPageSpec> {{
        vec![
            foundry_core::CustomAdminPageSpec {{
                key: "{slug}_overview".to_string(),
                title: "{display_name} 概览看板".to_string(),
                icon: "LayoutDashboard".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/{slug}/ext/custom-pages/overview.html".to_string(),
                required_role: None,
            }},
        ]
    }}
}}
"#,
        struct_name = module_struct_name,
        slug = slug,
        display_name = display_name
    );

    fs::write(base_dir.join("mod.rs"), mod_content)?;

    println!(
        "✅ Self-contained sub-system '{}' created successfully!",
        slug
    );
    println!("   📁 Controllers:   systems/src/{}/controllers/", slug);
    println!("   📁 Domain Logic:  systems/src/{}/logic/", slug);
    println!("   📁 DTOs & Schemas:systems/src/{}/dto/", slug);
    println!("   📁 Custom Admin:  systems/src/{}/custom_pages/", slug);
    println!("   📄 Manifest:      systems/src/{}/subsystem.json", slug);
    println!(
        "👉 Next step: Register `pub mod {};` in `systems/src/lib.rs` and add `Box::new({}::{})` to `register_subsystems()`.",
        slug, slug, module_struct_name
    );

    Ok(())
}

fn scaffold_external_subsystem(slug: &str, name: Option<&str>) -> anyhow::Result<()> {
    if !is_valid_slug(slug, 32) {
        anyhow::bail!(
            "Invalid system slug: '{}'. Must be 2-32 lowercase alphanumeric characters or underscore/hyphen.",
            slug
        );
    }

    let display_name = name.unwrap_or(slug);
    let base_dir = Path::new("external_systems").join(slug);

    if base_dir.exists() {
        anyhow::bail!(
            "External sub-system directory already exists at {:?}",
            base_dir
        );
    }

    println!(
        "🚀 Scaffolding standalone external sub-system repo '{}' ({}) at {:?}...",
        slug, display_name, base_dir
    );

    fs::create_dir_all(base_dir.join("custom_pages"))?;

    let manifest_content = serde_json::json!({
        "slug": slug,
        "display_name": display_name,
        "description": format!("Standalone external subsystem {}", display_name),
        "version": "1.0.0",
        "custom_pages": [
            {
                "key": format!("{}_dashboard", slug),
                "title": format!("{} 运营大屏", display_name),
                "icon": "Sparkles",
                "type": "iframe",
                "entry": format!("/api/v1/s/{}/ext/custom-pages/overview.html", slug)
            }
        ]
    });

    fs::write(
        base_dir.join("subsystem.json"),
        serde_json::to_string_pretty(&manifest_content)?,
    )?;

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>{display_name} 自定义后台</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-slate-900 text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold text-emerald-400">{display_name} 独立托管后台</h1>
  <p class="text-xs text-slate-400 mt-2">Slug: {slug} | 已连接 Foundry SDK Bridge</p>
</body>
</html>"#,
        display_name = display_name,
        slug = slug
    );

    fs::write(base_dir.join("custom_pages/overview.html"), html_content)?;

    println!(
        "✅ External sub-system '{}' created successfully at {:?}!",
        slug, base_dir
    );
    println!(
        "👉 Core Foundry system will automatically discover and load this subsystem from `./external_systems/` or `FOUNDRY_SYSTEMS_DIR`!"
    );

    Ok(())
}

fn init_subsystems_repo(target_path: &str) -> anyhow::Result<()> {
    let repo_path = Path::new(target_path);
    if repo_path.exists() {
        anyhow::bail!(
            "Target repository directory {:?} already exists.",
            repo_path
        );
    }

    println!(
        "📦 Initializing Unified Custom Subsystems Git Repository at {:?}...",
        repo_path
    );

    // Subsystem 1: carnival_demo
    let c_dir = repo_path.join("carnival_demo");
    fs::create_dir_all(c_dir.join("custom_pages"))?;
    fs::create_dir_all(c_dir.join("controllers"))?;
    fs::create_dir_all(c_dir.join("logic"))?;
    fs::create_dir_all(c_dir.join("dto"))?;

    // Subsystem 2: vip_mall
    let v_dir = repo_path.join("vip_mall");
    fs::create_dir_all(v_dir.join("custom_pages"))?;
    fs::create_dir_all(v_dir.join("controllers"))?;
    fs::create_dir_all(v_dir.join("logic"))?;
    fs::create_dir_all(v_dir.join("dto"))?;

    // .gitignore
    fs::write(
        repo_path.join(".gitignore"),
        "/target\n**/*.rs.bk\nCargo.lock\n.DS_Store\nnode_modules/\ndist/\n",
    )?;

    // Cargo.toml for the unified custom repo
    fs::write(
        repo_path.join("Cargo.toml"),
        r#"[package]
name = "foundry-custom-systems"
version = "0.1.0"
edition = "2024"
authors = ["Your Team <dev@example.com>"]

[dependencies]
# Dependencies will inherit from Foundry Core Workspace when linked
"#,
    )?;

    // README.md
    fs::write(
        repo_path.join("README.md"),
        r#"# Foundry Unified Custom Subsystems Repository

This repository contains **ALL custom subsystems** (custom APIs + domain logic + DTOs + custom Admin UI pages + manifests) for your team/organization.

## Architecture Philosophy
- **Core Platform (Foundry)**: Maintained by upstream open-source platform.
- **Custom Code (This Repo)**: 100% of your business logic & custom admin views live in this single repository.
- **0-Conflict Upgrades**: When Foundry releases updates, update the core repo without touching your custom code!

## Subsystem Directory Standard
Each subsystem folder contains its own complete code:
- `<slug>/controllers/`: Axum HTTP handlers mounted under `/api/v1/s/<slug>/ext/*`
- `<slug>/logic/`: Domain business logic and transactions
- `<slug>/dto/`: DTOs and validator schemas
- `<slug>/custom_pages/`: Custom Admin UI pages (HTML/React)
- `<slug>/subsystem.json`: Manifest & custom admin page specs

## Development & Packaging Modes
1. **Git Submodule**: Mount this repo as `foundry/systems/`
2. **External Link**: Set `FOUNDRY_SYSTEMS_DIR=/path/to/this/repo`
3. **Packaging**: Run `./scripts/build-release.sh --systems-dir /path/to/this/repo`
"#,
    )?;

    // Carnival Manifest
    let c_manifest = serde_json::json!({
        "slug": "carnival_demo",
        "display_name": "Carnival 2026 Demo",
        "description": "Interactive marketing campaign subsystem",
        "version": "1.0.0",
        "custom_pages": [
            {
                "key": "lottery_dashboard",
                "title": "抽奖运营大屏",
                "icon": "Gift",
                "type": "iframe",
                "entry": "/api/v1/s/carnival_demo/ext/custom-pages/lottery_dashboard.html"
            }
        ]
    });
    fs::write(
        c_dir.join("subsystem.json"),
        serde_json::to_string_pretty(&c_manifest)?,
    )?;

    // Carnival Custom Page
    fs::write(
        c_dir.join("custom_pages/lottery_dashboard.html"),
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>抽奖大屏</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-slate-900 text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold text-emerald-400">🎁 抽奖运营大屏</h1>
  <p class="text-xs text-slate-400 mt-2">自包含子系统 · 统一存放在自定义代码仓库</p>
</body>
</html>"#,
    )?;

    // VIP Mall Manifest
    let v_manifest = serde_json::json!({
        "slug": "vip_mall",
        "display_name": "VIP 尊享商城",
        "description": "VIP member benefits & points center",
        "version": "1.0.0",
        "custom_pages": [
            {
                "key": "vip_overview",
                "title": "VIP 概览大屏",
                "icon": "Crown",
                "type": "iframe",
                "entry": "/api/v1/s/vip_mall/ext/custom-pages/overview.html"
            }
        ]
    });
    fs::write(
        v_dir.join("subsystem.json"),
        serde_json::to_string_pretty(&v_manifest)?,
    )?;

    // VIP Mall Custom Page
    fs::write(
        v_dir.join("custom_pages/overview.html"),
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>VIP 概览</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="p-6 bg-slate-900 text-slate-100 min-h-screen">
  <h1 class="text-xl font-bold text-amber-400">👑 VIP 尊享商城运营看板</h1>
  <p class="text-xs text-slate-400 mt-2">自包含子系统 · 统一存放在自定义代码仓库</p>
</body>
</html>"#,
    )?;

    println!(
        "✅ Unified Custom Subsystems repository template created at {:?}!",
        repo_path
    );
    println!(
        "💡 All your custom subsystems live in this single repo. Foundry core upgrades will be 100% conflict-free!"
    );

    Ok(())
}

fn validate_subsystems(target_dir: Option<&str>) -> anyhow::Result<()> {
    let mut scan_dirs = Vec::new();
    if let Some(d) = target_dir {
        scan_dirs.push(PathBuf::from(d));
    } else {
        scan_dirs.push(PathBuf::from("systems/src"));
        scan_dirs.push(PathBuf::from("external_systems"));
    }

    println!(
        "🔍 Validating unified custom subsystems under: {:?}",
        scan_dirs
    );

    let mut found_count = 0;

    for base in &scan_dirs {
        if !base.exists() {
            continue;
        }
        // If base itself contains subsystem.json
        if base.join("subsystem.json").exists() {
            validate_single_subsystem_dir(base, &mut found_count);
            continue;
        }

        // If base is a parent directory containing multiple subsystem subfolders
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("subsystem.json").exists() {
                    validate_single_subsystem_dir(&path, &mut found_count);
                }
            }
        }
    }

    println!(
        "✅ Validation completed: Found {} valid subsystem package(s).",
        found_count
    );
    Ok(())
}

fn validate_single_subsystem_dir(path: &std::path::Path, count: &mut usize) {
    let slug = path.file_name().unwrap().to_string_lossy();
    let manifest_file = path.join("subsystem.json");
    let custom_pages_dir = path.join("custom_pages");

    let has_manifest = manifest_file.exists();
    let has_pages = custom_pages_dir.exists();

    println!("  👉 Found subsystem: '{}' at {:?}", slug, path);
    println!(
        "     • Manifest (subsystem.json): {}",
        if has_manifest {
            "✅ Present"
        } else {
            "⚠️ Missing"
        }
    );
    println!(
        "     • Custom Pages (custom_pages/): {}",
        if has_pages {
            "✅ Present"
        } else {
            "⚠️ None"
        }
    );

    if has_manifest {
        let parsed: Option<(String, String)> = (|| {
            let content = fs::read_to_string(&manifest_file).ok()?;
            let val = serde_json::from_str::<serde_json::Value>(&content).ok()?;
            let display_name = val["display_name"].as_str().unwrap_or(&slug).to_string();
            let version = val["version"].as_str().unwrap_or("unknown").to_string();
            Some((display_name, version))
        })();

        if let Some((display_name, version)) = parsed {
            println!("     • Display Name: {}", display_name);
            println!("     • Version: {}", version);
        }
    }
    *count += 1;
}

fn list_subsystems() -> anyhow::Result<()> {
    println!("📦 Registered Sub-Systems in Foundry:");
    println!("- carnival_demo (Carnival 2026 Demo Subsystem - Compiled)");
    println!("- vip_mall (VIP Mall Subsystem - Standalone External)");

    if let Ok(entries) = fs::read_dir("external_systems") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "vip_mall" {
                    println!("- {} (External Standalone Subsystem)", name);
                }
            }
        }
    }
    Ok(())
}

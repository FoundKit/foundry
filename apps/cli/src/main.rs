use clap::{Parser, Subcommand};
use foundry_auth::hash_password;
use foundry_core::types::is_valid_slug;
use foundry_storage::{AdminStore, init_db_pool, run_migrations};
use std::fs;
use std::path::Path;

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
        /// Unique sub-system slug (e.g., carnival_2026, vip_mall)
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
            SystemCommands::List => {
                println!("📦 Registered Sub-Systems in Foundry:");
                println!("- carnival_demo (Carnival 2026 Demo Subsystem - Compiled)");
                println!("- vip_mall (VIP Mall Subsystem - Standalone External)");
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
        "🚀 Scaffolding new sub-system '{}' ({}) at {:?}...",
        slug, display_name, base_dir
    );

    fs::create_dir_all(base_dir.join("controllers"))?;
    fs::create_dir_all(base_dir.join("logic"))?;
    fs::create_dir_all(base_dir.join("dto"))?;

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
        r#"use crate::"#.to_string()
            + slug
            + r#"::dto::{SampleRequest, SampleResponse};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;

pub struct SampleService;

impl SampleService {
    pub async fn execute(_ctx: &SystemContext, req: SampleRequest) -> AppResult<SampleResponse> {
        Ok(SampleResponse {
            message: format!("Processed request: {}", req.query),
        })
    }
}
"#,
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
        r#"use axum::{extract::Extension, Json};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::ApiResponse;
use validator::Validate;
use crate::"#
            .to_string()
            + slug
            + r#"::dto::{SampleRequest, SampleResponse};
use crate::"#
            + slug
            + r#"::logic::SampleService;

pub async fn handle_sample(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<SampleRequest>,
) -> AppResult<Json<ApiResponse<SampleResponse>>> {
    payload.validate()?;
    let result = SampleService::execute(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}
"#,
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

use axum::Router;
use foundry_core::SubsystemModule;

pub struct {struct_name};

impl SubsystemModule for {struct_name} {{
    fn slug(&self) -> &'static str {{
        "{slug}"
    }}

    fn display_name(&self) -> &'static str {{
        "{display_name}"
    }}

    fn register_routes(&self, router: Router) -> Router {{
        router.merge(controllers::build_routes())
    }}
}}
"#,
        struct_name = module_struct_name,
        slug = slug,
        display_name = display_name
    );

    fs::write(base_dir.join("mod.rs"), mod_content)?;

    println!("✅ Sub-system '{}' created successfully!", slug);
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
        anyhow::bail!("External sub-system directory already exists at {:?}", base_dir);
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
                "entry": format!("/custom-pages/{}/index.html", slug)
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
<body class="p-6 bg-slate-900 text-slate-100">
  <h1 class="text-xl font-bold text-emerald-400">{display_name} 独立托管后台</h1>
  <p class="text-xs text-slate-400 mt-2">Slug: {slug} | 已连接 Foundry SDK Bridge</p>
</body>
</html>"#,
        display_name = display_name,
        slug = slug
    );

    fs::write(base_dir.join("custom_pages/index.html"), html_content)?;

    println!("✅ External sub-system '{}' created successfully at {:?}!", slug, base_dir);
    println!("👉 Core Foundry system will automatically discover and load this subsystem from `./external_systems/` or `FOUNDRY_SYSTEMS_DIR`!");

    Ok(())
}


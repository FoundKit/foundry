use std::fs;
use std::process::Command;

#[test]
fn test_cli_new_project_compilation_smoke_test() {
    let temp_dir = std::env::temp_dir().join(format!("foundry_smoke_app_{}", uuid::Uuid::new_v4()));
    let project_name = temp_dir.to_string_lossy().to_string();

    // Get the absolute path to crates/foundry
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let foundry_crate_path = manifest_dir.join("../foundry");

    println!("Scaffolding smoke test application at {:?}", temp_dir);
    foundry_cli::scaffold_project(
        &project_name,
        foundry_cli::ProjectOptions {
            local_path: Some(foundry_crate_path.canonicalize().unwrap().to_str().unwrap()),
            db: Some("postgres"),
            ..Default::default()
        },
    )
    .expect("Failed to scaffold new project");

    assert!(temp_dir.join("Cargo.toml").exists());
    assert!(temp_dir.join("src/main.rs").exists());
    assert!(!temp_dir.join("dev").exists());
    assert!(!temp_dir.join("dev/docker-compose.yml").exists());
    assert!(!temp_dir.join("config").exists());
    assert!(temp_dir.join(".gitignore").exists());
    let gitignore = fs::read_to_string(temp_dir.join(".gitignore")).unwrap();
    assert!(!gitignore.contains("dev/"));
    assert!(temp_dir.join("README.md").exists());
    let readme = fs::read_to_string(temp_dir.join("README.md")).unwrap();
    assert!(!readme.contains("dev/docker-compose.yml"));
    assert!(readme.contains("foundry system new"));
    assert!(readme.contains("RecordStore"));
    assert!(readme.contains("单体模块化"));
    assert!(readme.contains("数据库初始化与管控说明"));
    assert!(readme.contains("GET http://localhost:8080/api/v1/health"));
    assert!(temp_dir.join("src/systems/sample/mod.rs").exists());

    // Verify .env and .env.example contain SMTP configuration variables
    assert!(temp_dir.join(".env.example").exists());
    assert!(temp_dir.join(".env").exists());
    let env_example = fs::read_to_string(temp_dir.join(".env.example")).unwrap();
    let env_content = fs::read_to_string(temp_dir.join(".env")).unwrap();
    assert!(env_example.contains("SMTP_HOST"));
    assert!(env_example.contains("SMTP_PORT"));
    assert!(env_example.contains("SMTP_USER"));
    assert!(env_example.contains("SMTP_PASS"));
    assert!(env_example.contains("SMTP_FROM"));
    assert!(env_content.contains("SMTP_HOST"));
    assert!(env_content.contains("SMTP_PORT"));
    assert!(env_content.contains("SMTP_USER"));
    assert!(env_content.contains("SMTP_PASS"));
    assert!(env_content.contains("SMTP_FROM"));

    // Verify migrations directory is absent by default
    assert!(!temp_dir.join("migrations").exists());

    // Verify on-demand generation creates migrations/init.sql
    let ddl_path = foundry_cli::generate_ddl(&temp_dir, foundry_cli::DatabaseKind::Postgres, None)
        .expect("Failed to generate postgres DDL");
    assert!(ddl_path.exists());
    assert!(temp_dir.join("migrations/init.sql").exists());
    let sql = fs::read_to_string(temp_dir.join("migrations/init.sql")).unwrap();
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS systems"));
    assert!(sql.contains("BIGSERIAL"));

    // Test scaffolding an additional subsystem
    foundry_cli::scaffold_subsystem("billing", Some("Billing Center"), &project_name)
        .expect("Failed to scaffold billing subsystem");
    assert!(temp_dir.join("src/systems/billing/mod.rs").exists());
    assert!(
        temp_dir
            .join("src/systems/billing/controllers/mod.rs")
            .exists()
    );
    assert!(temp_dir.join("src/systems/billing/logic/mod.rs").exists());
    assert!(temp_dir.join("src/systems/billing/dto/mod.rs").exists());
    assert!(
        temp_dir
            .join("src/systems/billing/custom_pages/overview.html")
            .exists()
    );

    // Run cargo check on the newly scaffolded project with multiple subsystems
    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&temp_dir)
        .status()
        .expect("Failed to run cargo check on generated project");

    assert!(status.success(), "Generated project cargo check failed");

    // Clean up
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cli_new_mysql_project_compilation_smoke_test() {
    let temp_dir =
        std::env::temp_dir().join(format!("foundry_smoke_mysql_{}", uuid::Uuid::new_v4()));
    let project_name = temp_dir.to_string_lossy().to_string();

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let foundry_crate_path = manifest_dir.join("../foundry");

    foundry_cli::scaffold_project(
        &project_name,
        foundry_cli::ProjectOptions {
            local_path: Some(foundry_crate_path.canonicalize().unwrap().to_str().unwrap()),
            db: Some("mysql"),
            ..Default::default()
        },
    )
    .expect("Failed to scaffold new mysql project");

    assert!(!temp_dir.join("dev").exists());
    assert!(!temp_dir.join("config").exists());
    // Verify migrations directory is absent by default
    assert!(!temp_dir.join("migrations").exists());

    // Verify on-demand generation creates migrations/init.sql
    let ddl_path = foundry_cli::generate_ddl(&temp_dir, foundry_cli::DatabaseKind::MySql, None)
        .expect("Failed to generate mysql DDL");
    assert!(ddl_path.exists());
    assert!(temp_dir.join("migrations/init.sql").exists());
    let sql = fs::read_to_string(temp_dir.join("migrations/init.sql")).unwrap();
    assert!(sql.contains("ENGINE=InnoDB"));
    assert!(sql.contains("MySQL"));

    let cargo_toml = fs::read_to_string(temp_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains(r#"features = ["standard", "mysql"]"#));

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&temp_dir)
        .status()
        .expect("Failed to run cargo check on mysql project");

    assert!(
        status.success(),
        "Generated mysql project cargo check failed"
    );

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cli_new_project_with_smtp_flags() {
    let temp_dir =
        std::env::temp_dir().join(format!("foundry_smoke_smtp_{}", uuid::Uuid::new_v4()));
    let project_name = temp_dir.to_string_lossy().to_string();

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let foundry_crate_path = manifest_dir.join("../foundry");

    foundry_cli::scaffold_project(
        &project_name,
        foundry_cli::ProjectOptions {
            local_path: Some(foundry_crate_path.canonicalize().unwrap().to_str().unwrap()),
            db: Some("postgres"),
            smtp_host: Some("smtp.custom.io"),
            smtp_port: Some(465),
            smtp_user: Some("mailer@custom.io"),
            smtp_pass: Some("secret123"),
            smtp_from: Some("Custom <mailer@custom.io>"),
            ..Default::default()
        },
    )
    .expect("Failed to scaffold new project with smtp options");

    assert!(temp_dir.join(".env").exists());
    let env_content = fs::read_to_string(temp_dir.join(".env")).unwrap();
    assert!(env_content.contains("SMTP_HOST=smtp.custom.io"));
    assert!(env_content.contains("SMTP_PORT=465"));
    assert!(env_content.contains("SMTP_USER=mailer@custom.io"));
    assert!(env_content.contains("SMTP_PASS=secret123"));
    assert!(env_content.contains("SMTP_FROM=Custom <mailer@custom.io>"));

    let _ = fs::remove_dir_all(temp_dir);
}

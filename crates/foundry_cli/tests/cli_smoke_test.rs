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
            ..Default::default()
        },
    )
    .expect("Failed to scaffold new project");

    assert!(temp_dir.join("Cargo.toml").exists());
    assert!(temp_dir.join("src/main.rs").exists());
    assert!(temp_dir.join("dev/docker-compose.yml").exists());
    let compose = fs::read_to_string(temp_dir.join("dev/docker-compose.yml")).unwrap();
    assert!(compose.contains("postgres:18-alpine"));
    assert!(compose.contains("postgres_data:/var/lib/postgresql\n"));
    assert!(temp_dir.join(".gitignore").exists());
    let gitignore = fs::read_to_string(temp_dir.join(".gitignore")).unwrap();
    assert!(gitignore.contains("dev/"));
    assert!(temp_dir.join("README.md").exists());
    let readme = fs::read_to_string(temp_dir.join("README.md")).unwrap();
    assert!(readme.contains("dev/docker-compose.yml"));
    assert!(readme.contains("foundry system new"));
    assert!(readme.contains("RecordStore"));
    assert!(readme.contains("单体模块化"));
    assert!(readme.contains("数据库运维与管控说明"));
    assert!(readme.contains("GET http://localhost:8080/api/v1/health"));
    assert!(!readme.contains("数据库初始化与权限控制说明"));
    assert!(!readme.contains("foundry admin create"));
    assert!(!readme.contains("foundry system new-external"));
    assert!(temp_dir.join("src/systems/sample/mod.rs").exists());
    assert!(temp_dir.join("migrations/init.sql").exists());
    assert!(temp_dir.join("dev/init-db.sh").exists());
    assert!(temp_dir.join("dev/init-dev-db.sql").exists());
    assert!(compose.contains("01_init.sql"));

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

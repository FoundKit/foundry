use foundry::prelude::*;
use foundry_core::orm::{ColumnDef, EntitySchema};

struct CustomBillingSubsystem;

impl SubsystemModule for CustomBillingSubsystem {
    fn slug(&self) -> &'static str {
        "billing"
    }

    fn display_name(&self) -> &'static str {
        "Billing Subsystem"
    }

    fn orm_models(&self) -> Vec<EntitySchema> {
        vec![
            EntitySchema::new("test_missing_billing_table")
                .column(ColumnDef::bigint("id").primary_key().auto_increment())
                .column(ColumnDef::varchar("invoice_no", 64).not_null()),
        ]
    }
}

#[tokio::test]
async fn test_schema_verification_fails_when_tables_missing_mysql() {
    let mysql_url = std::env::var("MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/foundry_dev".to_string());

    // Check if MySQL is accessible
    if sqlx::MySqlPool::connect(&mysql_url).await.is_err() {
        eprintln!("Skipping test: MySQL not accessible at {}", mysql_url);
        return;
    }

    // foundry_dev in MySQL is unmigrated (0 tables), so startup with auto_migrate=false must fail
    let result = FoundryApp::builder()
        .database_url(&mysql_url)
        .auto_migrate(false)
        .init_tracing(false)
        .build()
        .await;

    assert!(
        result.is_err(),
        "Expected startup to fail when database schema is incomplete"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Database schema verification failed for driver 'mysql'"),
        "Unexpected error message: {}",
        err_msg
    );
    assert!(
        err_msg.contains("systems") && err_msg.contains("admins"),
        "Missing tables not listed in error message: {}",
        err_msg
    );
    assert!(
        err_msg.contains("AUTO_MIGRATE=false"),
        "Error message should mention AUTO_MIGRATE=false: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_schema_verification_succeeds_when_complete_postgres() {
    foundry_storage::clear_registered_orm_models();
    let pg_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/foundry_dev".to_string());

    // Check if Postgres is accessible
    let pool = match sqlx::PgPool::connect(&pg_url).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not accessible at {}", pg_url);
            return;
        }
    };

    // Clean up any temporary test table
    let _ = sqlx::query("DROP TABLE IF EXISTS test_missing_billing_table;")
        .execute(&pool)
        .await;

    // Ensure baseline migrations are applied first
    let _ = foundry_storage::Database::run_migrations(&pg_url, Some("postgres")).await;

    // With complete schema and auto_migrate = false, startup must succeed
    let result = FoundryApp::builder()
        .database_url(&pg_url)
        .auto_migrate(false)
        .init_tracing(false)
        .build()
        .await;

    assert!(
        result.is_ok(),
        "Expected startup to succeed when schema is complete: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_skip_schema_validation_escape_hatch() {
    let mysql_url = std::env::var("MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/foundry_dev".to_string());

    if sqlx::MySqlPool::connect(&mysql_url).await.is_err() {
        eprintln!("Skipping test: MySQL not accessible at {}", mysql_url);
        return;
    }

    // When skip_schema_validation is true, unmigrated database startup does not fail verification
    let result = FoundryApp::builder()
        .database_url(&mysql_url)
        .auto_migrate(false)
        .skip_schema_validation(true)
        .init_tracing(false)
        .build()
        .await;

    assert!(
        result.is_ok(),
        "Expected startup to succeed when skip_schema_validation is enabled: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_custom_subsystem_orm_model_missing_detected() {
    foundry_storage::clear_registered_orm_models();
    let pg_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/foundry_dev".to_string());

    let pool = match sqlx::PgPool::connect(&pg_url).await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not accessible at {}", pg_url);
            return;
        }
    };

    // Ensure the test table does not exist
    let _ = sqlx::query("DROP TABLE IF EXISTS test_missing_billing_table;")
        .execute(&pool)
        .await;

    // Register a subsystem that requires a model that does not exist in the database
    let result = FoundryApp::builder()
        .database_url(&pg_url)
        .auto_migrate(false)
        .init_tracing(false)
        .register_subsystem(CustomBillingSubsystem)
        .build()
        .await;

    // Clean up registered models so other tests aren't affected
    foundry_storage::clear_registered_orm_models();

    assert!(
        result.is_err(),
        "Expected startup to fail when subsystem ORM table is missing"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("test_missing_billing_table"),
        "Expected error message to mention 'test_missing_billing_table', got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_direct_verify_schema_api() {
    let mysql_url = std::env::var("MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/foundry_dev".to_string());

    if sqlx::MySqlPool::connect(&mysql_url).await.is_err() {
        eprintln!("Skipping test: MySQL not accessible at {}", mysql_url);
        return;
    }

    let result = foundry_storage::Database::verify_schema(&mysql_url, Some("mysql")).await;
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("Database schema verification failed"));
}

#[tokio::test]
async fn test_schema_verification_fails_when_tables_missing_mariadb() {
    let mariadb_url = std::env::var("MARIADB_URL")
        .unwrap_or_else(|_| "mariadb://root:root@127.0.0.1:3307/foundry_dev".to_string());

    let normalized = mariadb_url.replace("mariadb://", "mysql://");
    if sqlx::MySqlPool::connect(&normalized).await.is_err() {
        eprintln!("Skipping test: MariaDB not accessible at {}", mariadb_url);
        return;
    }

    let result = FoundryApp::builder()
        .database_url(&mariadb_url)
        .auto_migrate(false)
        .init_tracing(false)
        .build()
        .await;

    assert!(
        result.is_err(),
        "Expected startup to fail when MariaDB schema is incomplete"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Database schema verification failed for driver 'mariadb'"),
        "Unexpected error: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_schema_verification_fails_when_collections_missing_mongodb() {
    let mongo_url = std::env::var("MONGODB_URL")
        .unwrap_or_else(|_| "mongodb://127.0.0.1:27017/foundry_dev".to_string());

    // Probe if MongoDB is running and accessible (skip quickly if unavailable in CI)
    let mut client_options =
        match foundry_storage::mongodb::options::ClientOptions::parse(&mongo_url).await {
            Ok(opt) => opt,
            Err(_) => {
                eprintln!("Skipping test: MongoDB URL parse failed");
                return;
            }
        };
    client_options.server_selection_timeout = Some(std::time::Duration::from_secs(2));
    client_options.connect_timeout = Some(std::time::Duration::from_secs(2));

    let client = match foundry_storage::mongodb::Client::with_options(client_options) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: MongoDB client creation failed");
            return;
        }
    };

    if client.list_database_names().await.is_err() {
        eprintln!("Skipping test: MongoDB not accessible at {}", mongo_url);
        return;
    }

    let result = FoundryApp::builder()
        .database_url(&mongo_url)
        .auto_migrate(false)
        .init_tracing(false)
        .build()
        .await;

    assert!(
        result.is_err(),
        "Expected startup to fail when MongoDB collections are missing"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Database schema verification failed for driver 'mongodb'"),
        "Unexpected error: {}",
        err_msg
    );
    assert!(
        err_msg.contains("systems") && err_msg.contains("admins"),
        "Expected missing collections in error: {}",
        err_msg
    );
}

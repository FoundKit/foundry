#[cfg(feature = "mariadb")]
mod mariadb_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use foundry_auth::JwtService;
    use foundry_core::response::ApiResponse;
    use foundry_engine::{AppState, build_router};
    use foundry_extension::HookPipeline;
    use foundry_storage::{AuditLogInsert, AuditLogQuery, Database, RecordQuery};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::ServiceExt;

    fn get_mariadb_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "mariadb://root:root@127.0.0.1:3307/foundry_mariadb_e2e_test".to_string()
        })
    }

    #[tokio::test]
    async fn test_mariadb_universal_storage_e2e() {
        let mariadb_url = get_mariadb_url();
        println!("Connecting to MariaDB for E2E testing: {}", mariadb_url);

        // Reset test database if possible for clean test state
        if let Ok(root_pool) = sqlx::MySqlPool::connect("mysql://root:root@127.0.0.1:3307/").await {
            let _ = sqlx::query("DROP DATABASE IF EXISTS foundry_mariadb_e2e_test;")
                .execute(&root_pool)
                .await;
            let _ = sqlx::query("CREATE DATABASE foundry_mariadb_e2e_test DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;").execute(&root_pool).await;
        }

        // 1. Connect to MariaDB with auto_migrate = true using mariadb:// scheme
        let db = match Database::connect(&mariadb_url, 5, true).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!(
                    "Skipping MariaDB E2E test - cannot connect to MariaDB: {}",
                    e
                );
                return;
            }
        };

        // 2. Admin Management & Authentication
        println!("Testing Admin creation and querying...");
        let admin_username = format!("superadmin_{}", uuid::Uuid::new_v4().simple());
        let admin = db
            .admins()
            .create(
                &admin_username,
                Some("admin@foundkit.org"),
                "$argon2id$v=19$m=19456,t=2,p=1$fake_hash_for_test",
                "superadmin",
                serde_json::json!(["*"]),
            )
            .await
            .expect("Failed to create admin in MariaDB");
        assert_eq!(admin.username, admin_username);

        let found_admin = db
            .admins()
            .get_by_username(&admin_username)
            .await
            .expect("Failed to retrieve admin by username");
        assert_eq!(found_admin.id, admin.id);

        // 3. System Management
        println!("Testing System creation and querying...");
        let sys_slug = format!("shop_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
        let system = db
            .systems()
            .create(&sys_slug, "E2E Test Shop", Some("Integration test system"))
            .await
            .expect("Failed to create system in MariaDB");
        assert_eq!(system.slug, sys_slug);

        let sys_item = db
            .systems()
            .get_by_slug(&sys_slug)
            .await
            .expect("Failed to get system by slug");
        assert_eq!(sys_item.slug, sys_slug);

        // 4. Model & Dynamic Fields Management
        println!("Testing Model & Field schema definitions...");
        let model = db
            .models()
            .create_model(
                &sys_slug,
                "products",
                "Product Catalog",
                Some("Test product catalog model"),
                None,
            )
            .await
            .expect("Failed to create model in MariaDB");
        assert_eq!(model.slug, "products");

        let f_title = db
            .models()
            .add_field(
                model.id,
                "title",
                "Product Title",
                "string",
                true,
                None,
                None,
                1,
            )
            .await
            .expect("Failed to add title field");
        assert_eq!(f_title.name, "title");

        let f_price = db
            .models()
            .add_field(model.id, "price", "Price", "number", true, None, None, 2)
            .await
            .expect("Failed to add price field");
        assert_eq!(f_price.name, "price");

        let f_stock = db
            .models()
            .add_field(
                model.id,
                "stock",
                "Stock Quantity",
                "integer",
                false,
                Some(serde_json::json!(0)),
                None,
                3,
            )
            .await
            .expect("Failed to add stock field");
        assert_eq!(f_stock.name, "stock");

        let fields = db
            .models()
            .list_fields(model.id)
            .await
            .expect("Failed to list fields");
        assert_eq!(fields.len(), 3);

        // 5. Universal Single-Table AutoCRUD with MariaDB
        println!("Testing Universal Single-Table AutoCRUD with MariaDB...");
        let record_1 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "Ergonomic Mechanical Keyboard",
                    "price": 129.99,
                    "stock": 50
                }),
            )
            .await
            .expect("Failed to insert record 1 in MariaDB");
        assert!(record_1.id > 0);
        assert_eq!(record_1.data["title"], "Ergonomic Mechanical Keyboard");

        let record_2 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "Wireless Gaming Mouse",
                    "price": 79.50,
                    "stock": 100
                }),
            )
            .await
            .expect("Failed to insert record 2 in MariaDB");
        assert!(record_2.id > record_1.id);

        let retrieved = db
            .records()
            .get_by_id(&sys_slug, "products", record_1.id)
            .await
            .expect("Failed to retrieve record by id");
        assert_eq!(retrieved.id, record_1.id);
        assert_eq!(retrieved.data["title"], "Ergonomic Mechanical Keyboard");

        let updated = db
            .records()
            .update(
                &sys_slug,
                "products",
                record_1.id,
                serde_json::json!({
                    "title": "Ergonomic Mechanical Keyboard (Pro V2)",
                    "price": 149.99,
                    "stock": 45
                }),
            )
            .await
            .expect("Failed to update record");
        assert_eq!(
            updated.data["title"],
            "Ergonomic Mechanical Keyboard (Pro V2)"
        );

        let list_res = db
            .records()
            .list(&sys_slug, "products", RecordQuery::default())
            .await
            .expect("Failed to list records");
        assert_eq!(list_res.pagination.total, 2);
        assert_eq!(list_res.items.len(), 2);

        // Soft delete record 2
        db.records()
            .delete(&sys_slug, "products", record_2.id)
            .await
            .expect("Failed to delete record 2");

        let list_after_delete = db
            .records()
            .list(&sys_slug, "products", RecordQuery::default())
            .await
            .expect("Failed to list records after delete");
        assert_eq!(list_after_delete.pagination.total, 1);
        assert_eq!(list_after_delete.items[0].id, record_1.id);

        // 6. Audit Logging in MariaDB
        println!("Testing Audit log recording...");
        db.audit()
            .insert(AuditLogInsert {
                admin_id: Some(admin.id),
                admin_username: Some(admin.username.clone()),
                system_slug: Some(sys_slug.clone()),
                method: "POST".to_string(),
                path: format!("/api/v1/s/{sys_slug}/products"),
                action_name: Some("CREATE_RECORD".to_string()),
                headers: serde_json::json!({"user-agent": "Foundry-Test-Agent"}),
                query_params: None,
                body_params: Some("{\"title\": \"Wireless Gaming Mouse\"}".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("Foundry-Test-Agent".to_string()),
                status_code: Some(200),
                duration_ms: Some(15),
            })
            .await
            .expect("Failed to insert audit log");

        let audit_logs = db
            .audit()
            .list(AuditLogQuery {
                system_slug: Some(sys_slug.clone()),
                ..Default::default()
            })
            .await
            .expect("Failed to query audit logs");
        assert_eq!(audit_logs.pagination.total, 1);
        assert_eq!(audit_logs.items[0].method, "POST");

        // 7. Full Axum Router HTTP integration with MariaDB storage
        println!("Testing full Axum HTTP router with MariaDB storage engine...");
        let jwt = JwtService::new("mariadb_super_secret_jwt_for_e2e_testing", 24);
        let token = jwt
            .generate_token(
                admin.id,
                &admin.username,
                "super_admin",
                vec!["*".to_string()],
            )
            .unwrap();

        let app_state = AppState::new(db.clone(), None, jwt, HookPipeline::new(), vec![]);
        let app = build_router(app_state);

        let health_req = Request::builder()
            .uri("/api/v1/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(health_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test Admin Systems Endpoint with JWT token
        let systems_req = Request::builder()
            .uri("/api/v1/admin/systems")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let sys_resp = app.clone().oneshot(systems_req).await.unwrap();
        assert_eq!(sys_resp.status(), StatusCode::OK);

        let body = sys_resp.into_body().collect().await.unwrap().to_bytes();
        let api_res: ApiResponse<Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(api_res.code, 0);

        println!("MariaDB Universal Single-Table E2E integration test succeeded perfectly!");
    }
}

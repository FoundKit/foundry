#[cfg(feature = "mysql")]
mod mysql_tests {
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

    fn get_mysql_url() -> String {
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:root@127.0.0.1:3306/foundry_e2e_test".to_string())
    }

    #[tokio::test]
    async fn test_mysql_universal_storage_e2e() {
        let mysql_url = get_mysql_url();
        println!("Connecting to MySQL for E2E testing: {}", mysql_url);

        // Reset test database if possible for clean test state
        if let Ok(root_pool) = sqlx::MySqlPool::connect("mysql://root:root@127.0.0.1:3306/").await {
            let _ = sqlx::query("DROP DATABASE IF EXISTS foundry_e2e_test;")
                .execute(&root_pool)
                .await;
            let _ = sqlx::query("CREATE DATABASE foundry_e2e_test DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;").execute(&root_pool).await;
        }

        // 1. Connect to MySQL with auto_migrate = true
        let db = match Database::connect(&mysql_url, 5, true).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Skipping MySQL E2E test - cannot connect to MySQL: {}", e);
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
            .expect("Failed to create admin in MySQL");
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
            .expect("Failed to create system in MySQL");
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
            .expect("Failed to create model in MySQL");
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

        let f_active = db
            .models()
            .add_field(
                model.id,
                "is_active",
                "Is Active",
                "boolean",
                false,
                Some(serde_json::json!(true)),
                None,
                4,
            )
            .await
            .expect("Failed to add is_active field");
        assert_eq!(f_active.name, "is_active");

        let fields = db
            .models()
            .list_fields(model.id)
            .await
            .expect("Failed to list fields");
        assert_eq!(fields.len(), 4);

        // 5. Universal Single-Table AutoCRUD Records
        println!("Testing Universal Single-Table AutoCRUD with MySQL...");
        // Test atomic re-fetch on insert
        let record1 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "Ergonomic Mechanical Keyboard",
                    "price": 129.99,
                    "stock": 50,
                    "is_active": true
                }),
            )
            .await
            .expect("Failed to insert record1 in MySQL");
        assert!(record1.id > 0);
        assert_eq!(record1.data["title"], "Ergonomic Mechanical Keyboard");
        assert_eq!(record1.data["price"], 129.99);

        let record2 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "Wireless Gaming Mouse",
                    "price": 79.50,
                    "stock": 100,
                    "is_active": true
                }),
            )
            .await
            .expect("Failed to insert record2 in MySQL");
        assert!(record2.id > record1.id);
        assert_eq!(record2.data["title"], "Wireless Gaming Mouse");

        // Test get by ID
        let fetched1 = db
            .records()
            .get_by_id(&sys_slug, "products", record1.id)
            .await
            .expect("Failed to fetch record1 by ID");
        assert_eq!(fetched1.id, record1.id);
        assert_eq!(fetched1.data["title"], "Ergonomic Mechanical Keyboard");

        // Test update
        let updated1 = db
            .records()
            .update(
                &sys_slug,
                "products",
                record1.id,
                serde_json::json!({
                    "title": "Ergonomic Mechanical Keyboard RGB Pro",
                    "price": 149.99,
                    "stock": 45,
                    "is_active": true
                }),
            )
            .await
            .expect("Failed to update record1");
        assert_eq!(
            updated1.data["title"],
            "Ergonomic Mechanical Keyboard RGB Pro"
        );
        assert_eq!(updated1.data["price"], 149.99);

        // Test Deferred Join pagination & covering index query
        let page1 = db
            .records()
            .list(
                &sys_slug,
                "products",
                RecordQuery {
                    page: Some(1),
                    page_size: Some(10),
                    sort_by: Some("id".to_string()),
                    sort_order: Some("desc".to_string()),
                },
            )
            .await
            .expect("Failed to list records with deferred join");
        assert_eq!(page1.pagination.total, 2);
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.items[0].id, record2.id); // sorted desc: record2 first
        assert_eq!(page1.items[1].id, record1.id);

        // Test Soft Delete
        db.records()
            .delete(&sys_slug, "products", record1.id)
            .await
            .expect("Failed to soft-delete record1");

        let page_after_delete = db
            .records()
            .list(&sys_slug, "products", RecordQuery::default())
            .await
            .expect("Failed to list records after deletion");
        assert_eq!(page_after_delete.pagination.total, 1);
        assert_eq!(page_after_delete.items[0].id, record2.id);

        // 6. Audit Log Recording & Querying
        println!("Testing Audit log recording...");
        db.audit()
            .insert(AuditLogInsert {
                admin_id: Some(admin.id),
                admin_username: Some(admin.username.clone()),
                system_slug: Some(sys_slug.clone()),
                method: "POST".to_string(),
                path: format!("/api/v1/s/{sys_slug}/products"),
                action_name: Some("CREATE_RECORD".to_string()),
                headers: serde_json::json!({ "user-agent": "MySQL-E2E-Tester" }),
                query_params: None,
                body_params: Some("{\"title\": \"Wireless Gaming Mouse\"}".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("MySQL-E2E-Tester".to_string()),
                status_code: Some(201),
                duration_ms: Some(12),
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
        assert_eq!(
            audit_logs.items[0].admin_username.as_deref(),
            Some(admin.username.as_str())
        );

        // 7. Full Axum Engine HTTP Integration with MySQL
        println!("Testing full Axum HTTP router with MySQL storage engine...");
        let jwt = JwtService::new("test_mysql_e2e_jwt_secret_123456", 24);
        let token = jwt
            .generate_token(
                admin.id,
                &admin.username,
                "superadmin",
                vec!["*".to_string()],
            )
            .expect("Failed to generate JWT token");

        let app_state = AppState::new(db.clone(), None, jwt, HookPipeline::new(), vec![]);
        let app = build_router(app_state);

        // Test GET /api/v1/health
        let health_req = Request::builder()
            .uri("/api/v1/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let health_res = app.clone().oneshot(health_req).await.unwrap();
        assert_eq!(health_res.status(), StatusCode::OK);

        // Test authenticated GET /api/v1/admin/systems
        let systems_req = Request::builder()
            .uri("/api/v1/admin/systems")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let systems_res = app.clone().oneshot(systems_req).await.unwrap();
        assert_eq!(systems_res.status(), StatusCode::OK);

        let body_bytes = systems_res.into_body().collect().await.unwrap().to_bytes();
        let resp_json: ApiResponse<Value> = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp_json.code, 0);

        // Test AutoCRUD GET /api/v1/s/:system_slug/:model_slug
        let records_req = Request::builder()
            .uri(format!("/api/v1/s/{sys_slug}/products"))
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let records_res = app.clone().oneshot(records_req).await.unwrap();
        assert_eq!(records_res.status(), StatusCode::OK);

        let rec_body = records_res.into_body().collect().await.unwrap().to_bytes();
        let rec_json: ApiResponse<Value> = serde_json::from_slice(&rec_body).unwrap();
        assert_eq!(rec_json.code, 0);
        let pagination_total = rec_json.data["pagination"]["total"].as_u64().unwrap();
        assert_eq!(pagination_total, 1);

        println!("MySQL Universal Single-Table E2E integration test succeeded perfectly!");
    }
}

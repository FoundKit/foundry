#[cfg(feature = "mongodb")]
mod mongodb_tests {
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

    fn get_mongodb_url() -> String {
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017/foundry_mongodb_e2e_test".to_string())
    }

    #[tokio::test]
    async fn test_mongodb_document_storage_e2e() {
        let mongodb_url = get_mongodb_url();
        println!("Connecting to MongoDB for E2E testing: {}", mongodb_url);

        // Reset test database if possible for clean test state
        if let Ok(client) =
            foundry_storage::mongodb::Client::with_uri_str("mongodb://127.0.0.1:27017").await
        {
            let _ = client.database("foundry_mongodb_e2e_test").drop().await;
        }

        // 1. Connect to MongoDB with auto_migrate = true
        let db = match Database::connect(&mongodb_url, 5, true).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!(
                    "Skipping MongoDB E2E test - cannot connect to MongoDB: {}",
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
            .expect("Failed to create admin in MongoDB");
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
            .create(
                &sys_slug,
                "MongoDB E2E Test Shop",
                Some("Document store integration system"),
            )
            .await
            .expect("Failed to create system in MongoDB");
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
                Some("MongoDB document catalog model"),
                None,
            )
            .await
            .expect("Failed to create model in MongoDB");
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

        // 5. Native Document AutoCRUD with MongoDB
        println!("Testing Native Document AutoCRUD with MongoDB...");
        let record_1 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "Cloud Native MongoDB Database",
                    "price": 299.00,
                    "stock": 100
                }),
            )
            .await
            .expect("Failed to insert record 1 in MongoDB");
        assert!(record_1.id > 0);
        assert_eq!(record_1.data["title"], "Cloud Native MongoDB Database");

        let record_2 = db
            .records()
            .create(
                &sys_slug,
                "products",
                serde_json::json!({
                    "title": "High Performance In-Memory Cache",
                    "price": 199.50,
                    "stock": 50
                }),
            )
            .await
            .expect("Failed to insert record 2 in MongoDB");
        assert!(record_2.id > record_1.id);

        let retrieved = db
            .records()
            .get_by_id(&sys_slug, "products", record_1.id)
            .await
            .expect("Failed to retrieve record by id");
        assert_eq!(retrieved.id, record_1.id);
        assert_eq!(retrieved.data["title"], "Cloud Native MongoDB Database");

        let updated = db
            .records()
            .update(
                &sys_slug,
                "products",
                record_1.id,
                serde_json::json!({
                    "title": "Cloud Native MongoDB Database (Cluster Edition)",
                    "price": 349.00,
                    "stock": 90
                }),
            )
            .await
            .expect("Failed to update record");
        assert_eq!(
            updated.data["title"],
            "Cloud Native MongoDB Database (Cluster Edition)"
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

        // 6. Audit Logging in MongoDB
        println!("Testing Audit log recording...");
        db.audit()
            .insert(AuditLogInsert {
                admin_id: Some(admin.id),
                admin_username: Some(admin.username.clone()),
                system_slug: Some(sys_slug.clone()),
                method: "POST".to_string(),
                path: format!("/api/s/{sys_slug}/products"),
                action_name: Some("CREATE_RECORD".to_string()),
                headers: serde_json::json!({"user-agent": "MongoDB-E2E-Tester"}),
                query_params: None,
                body_params: Some("{\"title\": \"High Performance In-Memory Cache\"}".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("MongoDB-E2E-Tester".to_string()),
                status_code: Some(200),
                duration_ms: Some(10),
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

        // 7. Full Axum Router HTTP integration with MongoDB storage
        println!("Testing full Axum HTTP router with MongoDB storage engine...");
        let jwt = JwtService::new("mongodb_super_secret_jwt_for_e2e_testing", 24);
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
            .uri("/api/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(health_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test Admin Systems Endpoint with JWT token
        let systems_req = Request::builder()
            .uri("/api/admin/systems")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let sys_resp = app.clone().oneshot(systems_req).await.unwrap();
        assert_eq!(sys_resp.status(), StatusCode::OK);

        let body = sys_resp.into_body().collect().await.unwrap().to_bytes();
        let api_res: ApiResponse<Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(api_res.code, 0);

        println!("MongoDB Document E2E integration test succeeded perfectly!");
    }
}

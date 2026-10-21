#[cfg(feature = "postgres")]
mod postgres_tests {
    use foundry_storage::{AuditLogInsert, AuditLogQuery, Database, RecordQuery};

    fn get_postgres_url() -> String {
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@127.0.0.1:5432/foundry_dev".to_string()
        })
    }

    #[tokio::test]
    async fn test_postgres_universal_storage_e2e() {
        let pg_url = get_postgres_url();
        println!("Connecting to PostgreSQL for E2E testing: {}", pg_url);

        // 1. Connect to PostgreSQL with auto_migrate = true
        let db = match Database::connect(&pg_url, 5, true).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!(
                    "Skipping PostgreSQL E2E test - cannot connect to PostgreSQL: {}",
                    e
                );
                return;
            }
        };

        // 2. Admin Management & Authentication
        println!("Testing Admin creation and querying...");
        let admin_username = format!("superadmin_{}", uuid::Uuid::new_v4().simple());
        let admin_email = format!("admin_{}@foundkit.org", uuid::Uuid::new_v4().simple());
        let admin = db
            .admins()
            .create(
                &admin_username,
                Some(&admin_email),
                "$argon2id$v=19$m=19456,t=2,p=1$fake_hash_for_test",
                "super_admin",
                serde_json::json!(["*"]),
            )
            .await
            .expect("Failed to create admin in PostgreSQL");
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
            .expect("Failed to create system in PostgreSQL");
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
            .expect("Failed to create model in PostgreSQL");
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

        // 5. Universal Single-Table AutoCRUD Records via ORM
        println!("Testing Universal Single-Table AutoCRUD with PostgreSQL ORM...");
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
            .expect("Failed to insert record1 in PostgreSQL");
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
            .expect("Failed to insert record2 in PostgreSQL");
        assert!(record2.id > record1.id);
        assert_eq!(record2.data["title"], "Wireless Gaming Mouse");

        // Test get by ID
        let fetched1 = db
            .records()
            .get_by_id(&sys_slug, "products", record1.id)
            .await
            .expect("Failed to fetch record1 by ID");
        assert_eq!(fetched1.id, record1.id);

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

        // Test list pagination
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
            .expect("Failed to list records");
        assert_eq!(page1.pagination.total, 2);
        assert_eq!(page1.items.len(), 2);

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
                path: format!("/api/s/{sys_slug}/products"),
                action_name: Some("CREATE_RECORD".to_string()),
                headers: serde_json::json!({ "user-agent": "PostgreSQL-E2E-Tester" }),
                query_params: None,
                body_params: Some("{\"title\": \"Wireless Gaming Mouse\"}".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("PostgreSQL-E2E-Tester".to_string()),
                status_code: Some(201),
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

        println!("PostgreSQL Universal Single-Table E2E integration test succeeded perfectly!");
    }
}

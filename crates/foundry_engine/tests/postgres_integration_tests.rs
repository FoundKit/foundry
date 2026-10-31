#[cfg(feature = "postgres")]
mod postgres_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use foundry_auth::JwtService;
    use foundry_core::response::ApiResponse;
    use foundry_engine::{AppState, build_router};
    use foundry_extension::HookPipeline;
    use foundry_storage::{
        AuditLogInsert, AuditLogQuery, Database, ModelRecordEntity, RecordQuery,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

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
                    ..Default::default()
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

        // 5.1 Test Model Modification (Renaming slug from products -> product_items)
        println!("Testing Model Modification (Renaming slug from products to product_items)...");
        let updated_model = db
            .models()
            .update_model(
                model.id,
                "product_items",
                "Product Catalog Updated",
                Some("Updated description"),
                None,
            )
            .await
            .expect("Failed to update model");
        assert_eq!(updated_model.slug, "product_items");
        assert_eq!(updated_model.name, "Product Catalog Updated");

        // Verify existing records are preserved and migrated to the new slug!
        let records_under_new_slug = db
            .records()
            .list(&sys_slug, "product_items", RecordQuery::default())
            .await
            .expect("Failed to list records under new slug");
        assert_eq!(records_under_new_slug.pagination.total, 1);
        assert_eq!(records_under_new_slug.items[0].id, record2.id);

        // 5.2 Test Field CRUD
        println!("Testing Field CRUD (Add, Update, Delete)...");
        let f_desc = db
            .models()
            .add_field(
                model.id,
                "description",
                "Product Description",
                "string",
                false,
                None,
                None,
                2,
            )
            .await
            .expect("Failed to add description field");
        assert_eq!(f_desc.name, "description");

        let f_desc_updated = db
            .models()
            .update_field(
                f_desc.id,
                "description",
                "Full Product Description",
                "richtext",
                true,
                None,
                None,
                2,
            )
            .await
            .expect("Failed to update description field");
        assert_eq!(f_desc_updated.label, "Full Product Description");
        assert_eq!(f_desc_updated.field_type, "richtext");

        db.models()
            .delete_field(f_desc.id)
            .await
            .expect("Failed to soft delete field");

        let active_fields = db
            .models()
            .list_fields(model.id)
            .await
            .expect("Failed to list active fields");
        assert_eq!(active_fields.len(), 1);
        assert_eq!(active_fields[0].name, "title");

        // 5.3 Test Model Deletion & Trash Bin (Soft Delete & Restore)
        println!("Testing Model Deletion & Trash Bin...");
        db.models()
            .delete_model_by_id(model.id)
            .await
            .expect("Failed to soft delete model by id");

        let active_models = db
            .models()
            .list_models(&sys_slug)
            .await
            .expect("Failed to list models");
        assert!(active_models.is_empty());

        let deleted_models = db
            .models()
            .list_deleted_models(&sys_slug)
            .await
            .expect("Failed to list deleted models");
        assert_eq!(deleted_models.len(), 1);
        assert_eq!(deleted_models[0].id, model.id);

        let restored_model = db
            .models()
            .restore_model(model.id)
            .await
            .expect("Failed to restore model");
        assert_eq!(restored_model.id, model.id);
        assert!(restored_model.deleted_at.is_none());

        let active_models_after_restore = db
            .models()
            .list_models(&sys_slug)
            .await
            .expect("Failed to list models after restore");
        assert_eq!(active_models_after_restore.len(), 1);
        assert_eq!(active_models_after_restore[0].id, model.id);

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

        // 7. Full HTTP End-to-End Test for Model Rename, Trash, Restore, and Obsolete Data Purging
        println!("Testing HTTP routes for Model Rename, Trash, Restore, and Data Purging...");
        let jwt = JwtService::new("test_secret_key_1234567890", 24);
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

        // 7.1 Create model 'tool' via HTTP POST /api/admin/s/:system_slug/models
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "tool",
                    "name": "Tool Model",
                    "description": "Initial tool model"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ModelEntity> = serde_json::from_slice(&body).unwrap();
        let tool_model = res.data;
        assert_eq!(tool_model.slug, "tool");

        // 7.2 Add field 'name' and field 'des'
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/s/{}/models/{}/fields",
                sys_slug, tool_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "name",
                    "label": "Tool Name",
                    "field_type": "string",
                    "is_required": true
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/s/{}/models/{}/fields",
                sys_slug, tool_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "des",
                    "label": "Description",
                    "field_type": "string",
                    "is_required": false
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ModelFieldEntity> =
            serde_json::from_slice(&body).unwrap();
        let des_field = res.data;

        // 7.3 Create a record under 'tool': {"name": "Hammer", "des": "For nails"}
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/s/{}/tool", sys_slug))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "Hammer",
                    "des": "For nails"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<ModelRecordEntity> = serde_json::from_slice(&body).unwrap();
        let record = res.data;
        assert_eq!(record.data["name"], "Hammer");
        assert_eq!(record.data["des"], "For nails");

        // 7.4 User renames model from 'tool' to 'tools'
        let req = Request::builder()
            .method("PUT")
            .uri(format!(
                "/api/admin/s/{}/models/{}",
                sys_slug, tool_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "tools",
                    "name": "Tools Collection",
                    "description": "Renamed to plural tools"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify the record can now be read under new slug '/api/s/:system_slug/tools/:id'
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/tools/{}", sys_slug, record.id))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<ModelRecordEntity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.data.id, record.id);
        assert_eq!(res.data.data["name"], "Hammer");

        // 7.5 User deletes field 'des' from model fields
        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/admin/s/{}/models/{}/fields/{}",
                sys_slug, tool_model.id, des_field.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 7.6 User updates the record: new value generated, 'des' is purged while 'name' is updated
        let req = Request::builder()
            .method("PUT")
            .uri(format!("/api/s/{}/tools/{}", sys_slug, record.id))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "Heavy Hammer",
                    "des": "Old obsolete des value"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<ModelRecordEntity> = serde_json::from_slice(&body).unwrap();
        let updated_record = res.data;
        assert_eq!(updated_record.data["name"], "Heavy Hammer");
        assert_eq!(
            updated_record.data.get("des"),
            None,
            "Obsolete field 'des' must be purged on update"
        );

        // 7.7 Soft delete model -> check trash -> restore model
        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/admin/s/{}/models/{}",
                sys_slug, tool_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify model is in trash
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/admin/s/{}/models/trash", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<Vec<foundry_storage::ModelEntity>> =
            serde_json::from_slice(&body).unwrap();
        assert!(res.data.iter().any(|m| m.id == tool_model.id));

        // Restore model
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/s/{}/models/{}/restore",
                sys_slug, tool_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify model is restored to active list
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<Vec<foundry_storage::ModelEntity>> =
            serde_json::from_slice(&body).unwrap();
        assert!(res.data.iter().any(|m| m.id == tool_model.id));

        // 7.8 Test Referential Integrity on Model Deletion, Export/Import, and Trash Uniqueness
        println!("Testing Model Referential Integrity, Trash Uniqueness, and Export/Import...");
        // Create authors model
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "authors",
                    "name": "Authors Model"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ModelEntity> = serde_json::from_slice(&body).unwrap();
        let authors_model = res.data;

        // Create books model
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "books",
                    "name": "Books Model"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ModelEntity> = serde_json::from_slice(&body).unwrap();
        let books_model = res.data;

        // Add author_ref relation field to books
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/s/{}/models/{}/fields",
                sys_slug, books_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "author_ref",
                    "label": "Author Reference",
                    "field_type": "relation",
                    "options": {
                        "target_system": sys_slug,
                        "target_model": "authors"
                    }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Attempt to delete 'authors' model -> MUST BE INTERCEPTED due to referential integrity!
        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/admin/s/{}/models/{}",
                sys_slug, authors_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let err_text = String::from_utf8_lossy(&body);
        assert!(
            err_text.contains("Books Model") && err_text.contains("author_ref"),
            "Error message should mention referencing model: {}",
            err_text
        );

        // Test Uniqueness check for trash bin:
        // Create a model 'temp_trash', delete it, then try creating 'temp_trash' again
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "temp_trash",
                    "name": "Temp Trash Model"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ModelEntity> = serde_json::from_slice(&body).unwrap();
        let temp_model = res.data;

        // Delete temp_trash
        let req = Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/admin/s/{}/models/{}",
                sys_slug, temp_model.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Try creating temp_trash again -> should fail with trash restore message
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "slug": "temp_trash",
                    "name": "Temp Trash Duplicate"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let err_text = String::from_utf8_lossy(&body);
        assert!(
            err_text.contains("回收站") || err_text.contains("trash"),
            "Expected trash restore message, got: {}",
            err_text
        );

        // Test Export models
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models/export", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "model_ids": [authors_model.id, books_model.id]
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<Vec<foundry_storage::ModelExportItem>> =
            serde_json::from_slice(&body).unwrap();
        let exported_items = res.data;
        assert_eq!(exported_items.len(), 2);

        // Test Import models into another subsystem
        let dest_sys_slug = format!("dest_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
        let _dest_sys = db
            .systems()
            .create(&dest_sys_slug, "Destination System", None)
            .await
            .expect("Failed to create destination system");

        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/models/import", dest_sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&exported_items).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify imported models in dest_sys_slug
        let dest_models = db.models().list_models(&dest_sys_slug).await.unwrap();
        assert_eq!(dest_models.len(), 2);
        let dest_books = dest_models.iter().find(|m| m.slug == "books").unwrap();
        let dest_books_fields = db.models().list_fields(dest_books.id).await.unwrap();
        let dest_author_ref = dest_books_fields
            .iter()
            .find(|f| f.name == "author_ref")
            .unwrap();
        assert_eq!(dest_author_ref.options["target_system"], dest_sys_slug);

        // 8. Application Management & Permission Control Tests
        println!("Testing Application Management & Scoped RBAC...");
        // 8.1 Create client application
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/s/{}/apps", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "Next.js SSR App",
                    "description": "Public Next.js frontend SSR client",
                    "permissions": {
                        "configs": { "read": true, "write": false },
                        "models": {
                            "tools": { "read": true, "create": false, "update": false, "delete": false }
                        }
                    }
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ApplicationEntity> =
            serde_json::from_slice(&body).unwrap();
        let client_app = res.data;
        assert_eq!(client_app.name, "Next.js SSR App");
        assert!(client_app.token.starts_with("sk_"));

        // 8.2 List applications for subsystem
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/admin/s/{}/apps", sys_slug))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<Vec<foundry_storage::ApplicationEntity>> =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(res.data.len(), 1);

        // 8.3 AutoCRUD call without Authorization header now rejected with 401 because apps exist
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/tools", sys_slug))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // 8.4 AutoCRUD call with invalid Bearer token rejected with 401
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/tools", sys_slug))
            .header("Authorization", "Bearer invalid_secret_token_123")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // 8.5 AutoCRUD call with valid app token reading 'tools' (permitted) succeeds
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/tools", sys_slug))
            .header("Authorization", format!("Bearer {}", client_app.token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 8.6 AutoCRUD call with valid app token creating 'tools' (forbidden) rejected with 403
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/s/{}/tools", sys_slug))
            .header("Authorization", format!("Bearer {}", client_app.token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "name": "Drill",
                    "des": "Power tool"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // 8.7 Configs read (permitted) succeeds
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/configs", sys_slug))
            .header("Authorization", format!("Bearer {}", client_app.token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 8.8 Configs write (forbidden) rejected with 403
        let req = Request::builder()
            .method("PUT")
            .uri(format!("/api/s/{}/configs", sys_slug))
            .header("Authorization", format!("Bearer {}", client_app.token))
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({
                    "site_title": "Hacked Title"
                }))
                .unwrap(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // 8.9 Regenerate app token
        let req = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/s/{}/apps/{}/regenerate-token",
                sys_slug, client_app.id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let res: ApiResponse<foundry_storage::ApplicationEntity> =
            serde_json::from_slice(&body).unwrap();
        assert_ne!(res.data.token, client_app.token);

        // Old token now rejected
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/s/{}/tools", sys_slug))
            .header("Authorization", format!("Bearer {}", client_app.token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // 8.10 Delete app
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/admin/s/{}/apps/{}", sys_slug, client_app.id))
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        println!("PostgreSQL Universal Single-Table E2E integration test succeeded perfectly!");
    }
}

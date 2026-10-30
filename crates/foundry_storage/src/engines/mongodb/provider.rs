use crate::engines::mongodb::records::MongoRecordEngine;
use crate::engines::mongodb::stores::{
    MongoAdminEngine, MongoApplicationEngine, MongoAuditEngine, MongoConfigEngine,
    MongoModelEngine, MongoSystemEngine,
};
use crate::spi::provider::{StorageDriverProvider, StorageEngineSet};
use async_trait::async_trait;
use foundry_core::error::{AppError, AppResult};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Database, IndexModel};
use std::sync::Arc;
use tracing::info;

pub struct MongoProvider;

#[async_trait]
impl StorageDriverProvider for MongoProvider {
    fn driver_name(&self) -> &'static str {
        "mongodb"
    }

    fn supports(&self, connection_url: &str) -> bool {
        connection_url.starts_with("mongodb://") || connection_url.starts_with("mongodb+srv://")
    }

    async fn create_engines(
        &self,
        connection_url: &str,
        _max_connections: u32,
    ) -> AppResult<StorageEngineSet> {
        let client_options = ClientOptions::parse(connection_url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to parse MongoDB URL: {}", e)))?;

        let db_name = client_options
            .default_database
            .clone()
            .unwrap_or_else(|| "foundry".to_string());

        let client = Client::with_options(client_options)
            .map_err(|e| AppError::Database(format!("Failed to create MongoDB client: {}", e)))?;

        let db = client.database(&db_name);

        Ok(StorageEngineSet {
            records: Arc::new(MongoRecordEngine::new(db.clone())),
            models: Arc::new(MongoModelEngine::new(db.clone())),
            systems: Arc::new(MongoSystemEngine::new(db.clone())),
            configs: Arc::new(MongoConfigEngine::new(db.clone())),
            admins: Arc::new(MongoAdminEngine::new(db.clone())),
            audit: Arc::new(MongoAuditEngine::new(db.clone())),
            apps: Arc::new(MongoApplicationEngine::new(db)),
            #[cfg(feature = "mysql")]
            mysql_pool: None,
            #[cfg(feature = "postgres")]
            pg_pool: None,
        })
    }

    async fn run_migrations(&self, connection_url: &str) -> AppResult<()> {
        info!("Running MongoDB collection and index initialization...");

        let client_options = ClientOptions::parse(connection_url).await.map_err(|e| {
            AppError::Database(format!("Failed to parse MongoDB URL for migrations: {}", e))
        })?;

        let db_name = client_options
            .default_database
            .clone()
            .unwrap_or_else(|| "foundry".to_string());

        let client = Client::with_options(client_options)
            .map_err(|e| AppError::Database(format!("Failed to create MongoDB client: {}", e)))?;

        let db = client.database(&db_name);

        // 1. Collections & Indexes
        init_mongodb_indexes(&db).await?;

        // 2. Default Initial Super Admin (admin / admin123456)
        let admin_coll = db.collection::<mongodb::bson::Document>("admins");
        let admin_exists = admin_coll
            .find_one(doc! { "username": "admin" })
            .await
            .map_err(|e| AppError::Database(format!("Check superadmin failed: {}", e)))?;

        if admin_exists.is_none() {
            let admin_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let now = chrono::Utc::now();

            let admin_doc = doc! {
                "id": admin_id.to_string(),
                "username": "admin",
                "email": "admin@foundry.local",
                "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q",
                "role": "super_admin",
                "allowed_systems": ["*"],
                "status": 1i32,
                "created_at": mongodb::bson::to_bson(&now).unwrap_or(mongodb::bson::Bson::Null),
                "updated_at": mongodb::bson::to_bson(&now).unwrap_or(mongodb::bson::Bson::Null),
            };

            admin_coll.insert_one(admin_doc).await.map_err(|e| {
                AppError::Database(format!("Failed to insert default super admin: {}", e))
            })?;
        }

        info!("MongoDB initialization completed successfully.");
        Ok(())
    }

    async fn get_existing_tables(&self, connection_url: &str) -> AppResult<Vec<String>> {
        if connection_url.contains("dummy") {
            return Ok(Vec::new());
        }

        let mut client_options = ClientOptions::parse(connection_url).await.map_err(|e| {
            AppError::Database(format!(
                "Failed to parse MongoDB URL for schema verification: {}",
                e
            ))
        })?;
        client_options.server_selection_timeout = Some(std::time::Duration::from_secs(3));
        client_options.connect_timeout = Some(std::time::Duration::from_secs(3));

        let db_name = client_options
            .default_database
            .clone()
            .unwrap_or_else(|| "foundry".to_string());

        let client = Client::with_options(client_options).map_err(|e| {
            AppError::Database(format!(
                "Failed to create MongoDB client for schema verification: {}",
                e
            ))
        })?;

        let db = client.database(&db_name);
        let names = db.list_collection_names().await.map_err(|e| {
            AppError::Database(format!(
                "Failed to query MongoDB collection metadata: {}",
                e
            ))
        })?;

        Ok(names)
    }

    fn get_baseline_ddl(&self) -> String {
        include_str!("../../../migrations/mongodb/init.js").to_string()
    }

    fn get_version_patch_ddl(&self, version: &str) -> Option<&'static str> {
        match version {
            "v0.2.0" | "0.2.0" => Some(include_str!(
                "../../../migrations/mongodb/versions/v0.2.0_add_applications.js"
            )),
            _ => None,
        }
    }

    fn default_migration_filename(&self) -> &'static str {
        "init.js"
    }
}

async fn init_mongodb_indexes(db: &Database) -> AppResult<()> {
    // systems: slug unique
    let systems = db.collection::<mongodb::bson::Document>("systems");
    let slug_idx = IndexModel::builder()
        .keys(doc! { "slug": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = systems.create_index(slug_idx).await;

    // models: (system_id, slug) unique
    let models = db.collection::<mongodb::bson::Document>("models");
    let model_unique = IndexModel::builder()
        .keys(doc! { "system_id": 1, "slug": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = models.create_index(model_unique).await;

    // model_fields: (model_id, name) unique
    let model_fields = db.collection::<mongodb::bson::Document>("model_fields");
    let field_unique = IndexModel::builder()
        .keys(doc! { "model_id": 1, "name": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = model_fields.create_index(field_unique).await;

    // system_configs: (system_id, key) unique
    let configs = db.collection::<mongodb::bson::Document>("system_configs");
    let config_unique = IndexModel::builder()
        .keys(doc! { "system_id": 1, "key": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = configs.create_index(config_unique).await;

    // model_records: query index + wildcard index on data.$**
    let records = db.collection::<mongodb::bson::Document>("model_records");
    let rec_query = IndexModel::builder()
        .keys(doc! { "model_id": 1, "deleted_at": 1, "created_at": -1, "id": -1 })
        .build();
    let _ = records.create_index(rec_query).await;

    let wildcard_idx = IndexModel::builder().keys(doc! { "data.$**": 1 }).build();
    let _ = records.create_index(wildcard_idx).await;

    // admins: username unique
    let admins = db.collection::<mongodb::bson::Document>("admins");
    let admin_user_idx = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = admins.create_index(admin_user_idx).await;

    // audit_logs: created_at index
    let audit = db.collection::<mongodb::bson::Document>("audit_logs");
    let audit_idx = IndexModel::builder()
        .keys(doc! { "created_at": -1 })
        .build();
    let _ = audit.create_index(audit_idx).await;

    // applications: (system_id, token unique)
    let apps = db.collection::<mongodb::bson::Document>("applications");
    let app_sys_idx = IndexModel::builder().keys(doc! { "system_id": 1 }).build();
    let _ = apps.create_index(app_sys_idx).await;

    let app_token_idx = IndexModel::builder()
        .keys(doc! { "token": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = apps.create_index(app_token_idx).await;

    Ok(())
}

use crate::engines::mongodb::counters::next_id;
use crate::entities::{
    AdminEntity, ApplicationEntity, AuditLogEntity, ModelEntity, ModelFieldEntity, PlatformSummary,
    SystemConfigEntity, SystemEntity, SystemItem, SystemStats,
};
use crate::traits::{
    AdminStoreEngine, ApplicationStoreEngine, AuditLogInsert, AuditLogQuery, AuditStoreEngine,
    ConfigStoreEngine, ModelStoreEngine, SystemQuery, SystemStoreEngine,
};
use async_trait::async_trait;
use chrono::Utc;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use mongodb::Database;
use mongodb::bson::doc;
use serde_json::Value;
use uuid::Uuid;

// ============================================================================
// MongoModelEngine
// ============================================================================

pub struct MongoModelEngine {
    db: Database,
}

impl MongoModelEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ModelStoreEngine for MongoModelEngine {
    async fn list_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! {
            "system_id": system_slug,
            "deleted_at": null,
        };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "id": 1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list models: {}", e)))?;

        let mut models = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Failed to deserialize model: {}", e)))?;
            let model: ModelEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Model mapping error: {}", e)))?;
            models.push(model);
        }
        Ok(models)
    }

    async fn list_deleted_models(&self, system_slug: &str) -> AppResult<Vec<ModelEntity>> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! {
            "system_id": system_slug,
            "deleted_at": { "$ne": null },
        };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "deleted_at": -1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list deleted models: {}", e)))?;

        let mut models = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Failed to deserialize model: {}", e)))?;
            let model: ModelEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Model mapping error: {}", e)))?;
            models.push(model);
        }
        Ok(models)
    }

    async fn get_model(&self, system_slug: &str, model_slug: &str) -> AppResult<ModelEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! {
            "system_id": system_slug,
            "slug": model_slug,
            "deleted_at": null,
        };
        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get model: {}", e)))?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Model '{}' not found in system '{}'",
                    model_slug, system_slug
                ))
            })?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Model mapping error: {}", e)))
    }

    async fn get_model_by_id(&self, model_id: i64) -> AppResult<ModelEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! { "id": model_id };
        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get model by id: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Model #{} not found", model_id)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Model mapping error: {}", e)))
    }

    async fn create_model(
        &self,
        system_slug: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        let default_perms = serde_json::json!({
            "public_read": false,
            "public_write": false,
            "auth_read": true,
            "auth_write": false
        });
        let perms = permissions.unwrap_or(default_perms);
        let new_id = next_id(&self.db, "models").await?;
        let now = Utc::now();

        let entity = ModelEntity {
            id: new_id,
            system_id: system_slug.to_string(),
            slug: slug.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            is_system: false,
            status: 1,
            permissions: perms,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Failed to serialize model: {}", e)))?;

        let coll = self.db.collection::<mongodb::bson::Document>("models");
        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Failed to insert model: {}", e)))?;

        Ok(entity)
    }

    async fn update_model(
        &self,
        model_id: i64,
        slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Option<Value>,
    ) -> AppResult<ModelEntity> {
        let current = self.get_model_by_id(model_id).await?;
        let coll = self.db.collection::<mongodb::bson::Document>("models");

        // If slug changed, ensure no other active model conflicts with the new slug
        if current.slug != slug {
            let conflict_filter = doc! {
                "system_id": &current.system_id,
                "slug": slug,
                "id": { "$ne": model_id },
                "deleted_at": null,
            };
            let conflict = coll.find_one(conflict_filter).await.map_err(|e| {
                AppError::Database(format!("Failed to check slug uniqueness: {}", e))
            })?;

            if conflict.is_some() {
                return Err(AppError::Validation(format!(
                    "A model with slug '{}' already exists in system '{}'",
                    slug, current.system_id
                )));
            }
        }

        let now = Utc::now();
        let now_bson = mongodb::bson::DateTime::from_millis(now.timestamp_millis());

        let mut set_doc = doc! {
            "slug": slug,
            "name": name,
            "updated_at": now_bson,
        };

        if let Some(desc) = description {
            set_doc.insert("description", desc);
        } else {
            set_doc.insert("description", mongodb::bson::Bson::Null);
        }

        if let Some(perms) = permissions {
            let perms_bson = mongodb::bson::to_bson(&perms).map_err(|e| {
                AppError::Database(format!("Failed to serialize permissions: {}", e))
            })?;
            set_doc.insert("permissions", perms_bson);
        }

        let filter = doc! { "id": model_id };
        let update = doc! { "$set": set_doc };

        coll.update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Failed to update model: {}", e)))?;

        // If slug changed, update model_records so existing data is preserved under the new slug
        if current.slug != slug {
            let records_coll = self
                .db
                .collection::<mongodb::bson::Document>("model_records");
            let rec_filter = doc! {
                "system_id": &current.system_id,
                "model_slug": &current.slug,
            };
            let rec_update = doc! {
                "$set": { "model_slug": slug, "updated_at": now_bson }
            };
            records_coll
                .update_many(rec_filter, rec_update)
                .await
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to migrate records to new model slug: {}",
                        e
                    ))
                })?;
        }

        self.get_model_by_id(model_id).await
    }

    async fn list_fields(&self, model_id: i64) -> AppResult<Vec<ModelFieldEntity>> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("model_fields");
        let filter = doc! { "model_id": model_id, "deleted_at": null };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "sort_order": 1, "id": 1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list fields: {}", e)))?;

        let mut fields = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Failed to deserialize field: {}", e)))?;
            let field: ModelFieldEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Field mapping error: {}", e)))?;
            fields.push(field);
        }
        Ok(fields)
    }

    async fn get_field(&self, field_id: i64) -> AppResult<ModelFieldEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("model_fields");
        let filter = doc! { "id": field_id, "deleted_at": null };
        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get field: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Field #{} not found", field_id)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Field mapping error: {}", e)))
    }

    async fn add_field(
        &self,
        model_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        let new_id = next_id(&self.db, "model_fields").await?;
        let now = Utc::now();
        let opts = options.unwrap_or_else(|| serde_json::json!({}));

        let entity = ModelFieldEntity {
            id: new_id,
            model_id,
            name: name.to_string(),
            label: label.to_string(),
            field_type: field_type.to_string(),
            is_required,
            default_value,
            options: opts,
            sort_order,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Failed to serialize field: {}", e)))?;

        let coll = self
            .db
            .collection::<mongodb::bson::Document>("model_fields");
        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Failed to insert field: {}", e)))?;

        Ok(entity)
    }

    async fn update_field(
        &self,
        field_id: i64,
        name: &str,
        label: &str,
        field_type: &str,
        is_required: bool,
        default_value: Option<Value>,
        options: Option<Value>,
        sort_order: i32,
    ) -> AppResult<ModelFieldEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("model_fields");
        let now = Utc::now();
        let now_bson = mongodb::bson::DateTime::from_millis(now.timestamp_millis());
        let opts = options.unwrap_or_else(|| serde_json::json!({}));
        let opts_bson = mongodb::bson::to_bson(&opts)
            .map_err(|e| AppError::Database(format!("Failed to serialize options: {}", e)))?;

        let default_val_bson = match default_value {
            Some(v) => mongodb::bson::to_bson(&v).map_err(|e| {
                AppError::Database(format!("Failed to serialize default_value: {}", e))
            })?,
            None => mongodb::bson::Bson::Null,
        };

        let filter = doc! { "id": field_id, "deleted_at": null };
        let update = doc! {
            "$set": {
                "name": name,
                "label": label,
                "field_type": field_type,
                "is_required": is_required,
                "default_value": default_val_bson,
                "options": opts_bson,
                "sort_order": sort_order,
                "updated_at": now_bson,
            }
        };

        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Failed to update field: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!("Field #{} not found", field_id)));
        }

        self.get_field(field_id).await
    }

    async fn delete_field(&self, field_id: i64) -> AppResult<()> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("model_fields");
        let filter = doc! { "id": field_id, "deleted_at": null };
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis()),
            }
        };

        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Failed to soft delete field: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!("Field #{} not found", field_id)));
        }

        Ok(())
    }

    async fn delete_model(&self, system_slug: &str, model_slug: &str) -> AppResult<()> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! {
            "system_id": system_slug,
            "slug": model_slug,
            "deleted_at": null,
        };
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis()),
            }
        };
        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Soft delete model failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!(
                "Model '{}' not found in system '{}'",
                model_slug, system_slug
            )));
        }

        Ok(())
    }

    async fn delete_model_by_id(&self, model_id: i64) -> AppResult<()> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! { "id": model_id, "deleted_at": null };
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis()),
            }
        };
        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Soft delete model failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!("Model #{} not found", model_id)));
        }

        Ok(())
    }

    async fn restore_model(&self, model_id: i64) -> AppResult<ModelEntity> {
        let model = self.get_model_by_id(model_id).await?;
        if model.deleted_at.is_none() {
            return Ok(model);
        }

        let coll = self.db.collection::<mongodb::bson::Document>("models");

        // Check if another active model exists with the same slug
        let conflict_filter = doc! {
            "system_id": &model.system_id,
            "slug": &model.slug,
            "id": { "$ne": model_id },
            "deleted_at": null,
        };
        let conflict = coll.find_one(conflict_filter).await.map_err(|e| {
            AppError::Database(format!("Failed to check slug uniqueness on restore: {}", e))
        })?;

        if conflict.is_some() {
            return Err(AppError::Validation(format!(
                "Cannot restore model: another active model with slug '{}' already exists in system '{}'",
                model.slug, model.system_id
            )));
        }

        let filter = doc! { "id": model_id };
        let now = Utc::now();
        let now_bson = mongodb::bson::DateTime::from_millis(now.timestamp_millis());
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::Bson::Null,
                "updated_at": now_bson,
            }
        };

        coll.update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Failed to restore model: {}", e)))?;

        self.get_model_by_id(model_id).await
    }
}

// ============================================================================
// MongoSystemEngine
// ============================================================================

pub struct MongoSystemEngine {
    db: Database,
}

impl MongoSystemEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SystemStoreEngine for MongoSystemEngine {
    async fn list(&self) -> AppResult<Vec<SystemEntity>> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let filter = doc! { "deleted_at": null };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": 1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list systems: {}", e)))?;

        let mut systems = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize system failed: {}", e)))?;
            let sys: SystemEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("System mapping error: {}", e)))?;
            systems.push(sys);
        }
        Ok(systems)
    }

    async fn list_paginated(
        &self,
        query: SystemQuery,
        allowed_systems: Option<&[String]>,
    ) -> AppResult<PaginatedData<SystemItem>> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let mut filter = doc! { "deleted_at": null };

        if let Some(allowed) = allowed_systems {
            filter.insert("slug", doc! { "$in": allowed });
        }

        if let Some(id) = query.id {
            filter.insert("id", id.to_string());
        }

        if let Some(ref slug) = query.slug {
            filter.insert("slug", doc! { "$regex": slug, "$options": "i" });
        }

        if let Some(ref name) = query.name {
            filter.insert("name", doc! { "$regex": name, "$options": "i" });
        }

        if let Some(ref kw) = query.keyword {
            filter.insert(
                "$or",
                vec![
                    doc! { "slug": { "$regex": kw, "$options": "i" } },
                    doc! { "name": { "$regex": kw, "$options": "i" } },
                    doc! { "description": { "$regex": kw, "$options": "i" } },
                ],
            );
        }

        if let Some(st) = query.status {
            filter.insert("status", st as i32);
        }

        let total = coll
            .count_documents(filter.clone())
            .await
            .map_err(|e| AppError::Database(format!("Count systems failed: {}", e)))?;

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(10).clamp(1, 100);
        let skip = (page - 1) * page_size;

        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(skip)
            .limit(page_size as i64)
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Fetch systems failed: {}", e)))?;

        let models_coll = self.db.collection::<mongodb::bson::Document>("models");
        let configs_coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let records_coll = self
            .db
            .collection::<mongodb::bson::Document>("model_records");

        let mut items = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize system failed: {}", e)))?;
            let sys: SystemEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("System mapping error: {}", e)))?;

            let models_count = models_coll
                .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
                .await
                .unwrap_or(0) as i64;

            let configs_count = configs_coll
                .count_documents(doc! { "system_id": &sys.slug })
                .await
                .unwrap_or(0) as i64;

            let records_count = records_coll
                .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
                .await
                .unwrap_or(0) as i64;

            items.push(SystemItem {
                id: sys.id,
                slug: sys.slug,
                name: sys.name,
                description: sys.description,
                status: sys.status,
                created_at: sys.created_at,
                updated_at: sys.updated_at,
                deleted_at: sys.deleted_at,
                models_count,
                configs_count,
                records_count,
            });
        }

        Ok(PaginatedData::new(items, page, page_size, total))
    }

    async fn get_by_id(&self, id: Uuid) -> AppResult<SystemItem> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let filter = doc! {
            "id": id.to_string(),
            "deleted_at": null,
        };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get system failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        let sys: SystemEntity = mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("System mapping error: {}", e)))?;

        let models_coll = self.db.collection::<mongodb::bson::Document>("models");
        let configs_coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let records_coll = self
            .db
            .collection::<mongodb::bson::Document>("model_records");

        let models_count = models_coll
            .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
            .await
            .unwrap_or(0) as i64;

        let configs_count = configs_coll
            .count_documents(doc! { "system_id": &sys.slug })
            .await
            .unwrap_or(0) as i64;

        let records_count = records_coll
            .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
            .await
            .unwrap_or(0) as i64;

        Ok(SystemItem {
            id: sys.id,
            slug: sys.slug,
            name: sys.name,
            description: sys.description,
            status: sys.status,
            created_at: sys.created_at,
            updated_at: sys.updated_at,
            deleted_at: sys.deleted_at,
            models_count,
            configs_count,
            records_count,
        })
    }

    async fn get_by_slug(&self, slug: &str) -> AppResult<SystemItem> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let filter = doc! {
            "slug": slug,
            "deleted_at": null,
        };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get system failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Sub-system '{}' not found", slug)))?;

        let sys: SystemEntity = mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("System mapping error: {}", e)))?;

        let models_coll = self.db.collection::<mongodb::bson::Document>("models");
        let configs_coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let records_coll = self
            .db
            .collection::<mongodb::bson::Document>("model_records");

        let models_count = models_coll
            .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
            .await
            .unwrap_or(0) as i64;

        let configs_count = configs_coll
            .count_documents(doc! { "system_id": &sys.slug })
            .await
            .unwrap_or(0) as i64;

        let records_count = records_coll
            .count_documents(doc! { "system_id": &sys.slug, "deleted_at": null })
            .await
            .unwrap_or(0) as i64;

        Ok(SystemItem {
            id: sys.id,
            slug: sys.slug,
            name: sys.name,
            description: sys.description,
            status: sys.status,
            created_at: sys.created_at,
            updated_at: sys.updated_at,
            deleted_at: sys.deleted_at,
            models_count,
            configs_count,
            records_count,
        })
    }

    async fn create(
        &self,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<SystemEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let now = Utc::now();
        let entity = SystemEntity {
            id: Uuid::new_v4(),
            slug: slug.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            status: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Serialize system failed: {}", e)))?;

        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Insert system failed: {}", e)))?;

        Ok(entity)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> AppResult<SystemEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let filter = doc! { "id": id.to_string(), "deleted_at": null };

        let doc_found = coll
            .find_one(filter.clone())
            .await
            .map_err(|e| AppError::Database(format!("Find system failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Sub-system #{} not found", id)))?;

        let mut sys: SystemEntity = mongodb::bson::from_document(doc_found)
            .map_err(|e| AppError::Database(format!("Deserialize system failed: {}", e)))?;

        if let Some(n) = name {
            sys.name = n.to_string();
        }
        if let Some(d) = description {
            sys.description = Some(d.to_string());
        }
        if let Some(st) = status {
            sys.status = st;
        }
        sys.updated_at = Utc::now();

        let updated_doc = mongodb::bson::to_document(&sys)
            .map_err(|e| AppError::Database(format!("Serialize system failed: {}", e)))?;

        coll.replace_one(filter, updated_doc)
            .await
            .map_err(|e| AppError::Database(format!("Update system failed: {}", e)))?;

        Ok(sys)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let coll = self.db.collection::<mongodb::bson::Document>("systems");
        let filter = doc! { "id": id.to_string(), "deleted_at": null };
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::DateTime::from_millis(chrono::Utc::now().timestamp_millis()),
            }
        };
        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Soft delete system failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!("Sub-system #{} not found", id)));
        }
        Ok(())
    }

    async fn get_stats(&self, slug: &str) -> AppResult<SystemStats> {
        let sys = self.get_by_slug(slug).await?;

        let audit_coll = self.db.collection::<mongodb::bson::Document>("audit_logs");
        let audit_logs_count = audit_coll
            .count_documents(doc! { "system_slug": slug })
            .await
            .unwrap_or(0);

        Ok(SystemStats {
            id: sys.id,
            slug: sys.slug,
            name: sys.name,
            description: sys.description,
            status: sys.status,
            created_at: sys.created_at,
            models_count: sys.models_count as u64,
            configs_count: sys.configs_count as u64,
            records_count: sys.records_count as u64,
            audit_logs_count,
        })
    }

    async fn platform_summary(&self) -> AppResult<PlatformSummary> {
        let systems_coll = self.db.collection::<mongodb::bson::Document>("systems");
        let models_coll = self.db.collection::<mongodb::bson::Document>("models");
        let records_coll = self
            .db
            .collection::<mongodb::bson::Document>("model_records");
        let admins_coll = self.db.collection::<mongodb::bson::Document>("admins");
        let audit_coll = self.db.collection::<mongodb::bson::Document>("audit_logs");

        let total_systems = systems_coll
            .count_documents(doc! { "deleted_at": null })
            .await
            .unwrap_or(0);

        let active_systems = systems_coll
            .count_documents(doc! { "deleted_at": null, "status": 1 })
            .await
            .unwrap_or(0);

        let total_models = models_coll
            .count_documents(doc! { "deleted_at": null })
            .await
            .unwrap_or(0);

        let total_records = records_coll
            .count_documents(doc! { "deleted_at": null })
            .await
            .unwrap_or(0);

        let total_admins = admins_coll
            .count_documents(doc! { "status": 1 })
            .await
            .unwrap_or(0);

        let total_audit_logs = audit_coll.count_documents(doc! {}).await.unwrap_or(0);

        Ok(PlatformSummary {
            total_systems,
            active_systems,
            total_models,
            total_records,
            total_admins,
            total_audit_logs,
        })
    }
}

// ============================================================================
// MongoConfigEngine
// ============================================================================

pub struct MongoConfigEngine {
    db: Database,
}

impl MongoConfigEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ConfigStoreEngine for MongoConfigEngine {
    async fn list(&self, system_slug: &str) -> AppResult<Vec<SystemConfigEntity>> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let filter = doc! { "system_id": system_slug };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "sort_order": 1, "id": 1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list configs: {}", e)))?;

        let mut configs = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize config failed: {}", e)))?;
            let cfg: SystemConfigEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Config mapping error: {}", e)))?;
            configs.push(cfg);
        }
        Ok(configs)
    }

    async fn get_aggregated(&self, system_slug: &str) -> AppResult<Value> {
        let configs = self.list(system_slug).await?;
        let mut map = serde_json::Map::new();

        for cfg in configs {
            let val = cfg.value.unwrap_or(Value::Null);
            map.insert(cfg.key, val);
        }

        Ok(Value::Object(map))
    }

    async fn upsert(
        &self,
        system_slug: &str,
        key: &str,
        label: &str,
        value_type: &str,
        value: Option<Value>,
        options: Value,
        sort_order: i32,
    ) -> AppResult<SystemConfigEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let filter = doc! { "system_id": system_slug, "key": key };

        let existing = coll
            .find_one(filter.clone())
            .await
            .map_err(|e| AppError::Database(format!("Find config failed: {}", e)))?;

        let now = Utc::now();
        if let Some(doc) = existing {
            let mut cfg: SystemConfigEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Deserialize config failed: {}", e)))?;
            cfg.label = label.to_string();
            cfg.value_type = value_type.to_string();
            cfg.value = value;
            cfg.options = options;
            cfg.sort_order = sort_order;
            cfg.updated_at = now;

            let updated_doc = mongodb::bson::to_document(&cfg)
                .map_err(|e| AppError::Database(format!("Serialize config failed: {}", e)))?;

            coll.replace_one(filter, updated_doc)
                .await
                .map_err(|e| AppError::Database(format!("Update config failed: {}", e)))?;

            Ok(cfg)
        } else {
            let new_id = next_id(&self.db, "system_configs").await?;
            let entity = SystemConfigEntity {
                id: new_id,
                system_id: system_slug.to_string(),
                key: key.to_string(),
                label: label.to_string(),
                value_type: value_type.to_string(),
                value,
                options,
                sort_order,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };

            let doc = mongodb::bson::to_document(&entity)
                .map_err(|e| AppError::Database(format!("Serialize config failed: {}", e)))?;

            coll.insert_one(doc)
                .await
                .map_err(|e| AppError::Database(format!("Insert config failed: {}", e)))?;

            Ok(entity)
        }
    }

    async fn update_values(
        &self,
        system_slug: &str,
        values: &serde_json::Map<String, Value>,
    ) -> AppResult<()> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("system_configs");
        let now = Utc::now();

        for (k, v) in values {
            let filter = doc! { "system_id": system_slug, "key": k };
            let bson_val = mongodb::bson::to_bson(v)
                .map_err(|e| AppError::Database(format!("Serialize config value failed: {}", e)))?;
            let update = doc! {
                "$set": {
                    "value": bson_val,
                    "updated_at": mongodb::bson::to_bson(&now).unwrap_or(mongodb::bson::Bson::Null),
                }
            };
            let _ = coll.update_one(filter, update).await;
        }

        Ok(())
    }
}

// ============================================================================
// MongoAdminEngine
// ============================================================================

pub struct MongoAdminEngine {
    db: Database,
}

impl MongoAdminEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AdminStoreEngine for MongoAdminEngine {
    async fn list(&self) -> AppResult<Vec<AdminEntity>> {
        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = coll
            .find(doc! {})
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list admins: {}", e)))?;

        let mut admins = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize admin failed: {}", e)))?;
            let admin: AdminEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Admin mapping error: {}", e)))?;
            admins.push(admin);
        }
        Ok(admins)
    }

    async fn get_by_id(&self, id: Uuid) -> AppResult<AdminEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let filter = doc! { "id": id.to_string() };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get admin by id failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Admin with id '{}' not found", id)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Admin mapping error: {}", e)))
    }

    async fn get_by_username(&self, username: &str) -> AppResult<AdminEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let filter = doc! { "username": username };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get admin by username failed: {}", e)))?
            .ok_or_else(|| {
                AppError::NotFound(format!("Admin with username '{}' not found", username))
            })?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Admin mapping error: {}", e)))
    }

    async fn get_by_email(&self, email: &str) -> AppResult<AdminEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let filter = doc! { "email": email };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get admin by email failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Admin with email '{}' not found", email)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Admin mapping error: {}", e)))
    }

    async fn create(
        &self,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: &str,
        allowed_systems: Value,
    ) -> AppResult<AdminEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let now = Utc::now();
        let entity = AdminEntity {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            password_hash: password_hash.to_string(),
            role: role.to_string(),
            allowed_systems,
            status: 1,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Serialize admin failed: {}", e)))?;

        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Insert admin failed: {}", e)))?;

        Ok(entity)
    }

    async fn update(
        &self,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
        allowed_systems: Option<Value>,
        status: Option<i16>,
    ) -> AppResult<AdminEntity> {
        let mut admin = self.get_by_id(id).await?;
        if let Some(em) = email {
            admin.email = Some(em.to_string());
        }
        if let Some(ph) = password_hash {
            admin.password_hash = ph.to_string();
        }
        if let Some(r) = role {
            admin.role = r.to_string();
        }
        if let Some(systems) = allowed_systems {
            admin.allowed_systems = systems;
        }
        if let Some(st) = status {
            admin.status = st;
        }
        admin.updated_at = Utc::now();

        let coll = self.db.collection::<mongodb::bson::Document>("admins");
        let filter = doc! { "id": id.to_string() };
        let updated_doc = mongodb::bson::to_document(&admin)
            .map_err(|e| AppError::Database(format!("Serialize admin failed: {}", e)))?;

        coll.replace_one(filter, updated_doc)
            .await
            .map_err(|e| AppError::Database(format!("Update admin failed: {}", e)))?;

        Ok(admin)
    }
}

// ============================================================================
// MongoAuditEngine
// ============================================================================

pub struct MongoAuditEngine {
    db: Database,
}

impl MongoAuditEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuditStoreEngine for MongoAuditEngine {
    async fn insert(&self, log: AuditLogInsert) -> AppResult<()> {
        let new_id = next_id(&self.db, "audit_logs").await?;
        let entity = AuditLogEntity {
            id: new_id,
            admin_id: log.admin_id,
            admin_username: log.admin_username,
            system_slug: log.system_slug,
            method: log.method,
            path: log.path,
            action_name: log.action_name,
            headers: log.headers,
            query_params: log.query_params,
            body_params: log.body_params,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
            status_code: log.status_code,
            duration_ms: log.duration_ms,
            created_at: Utc::now(),
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Serialize audit log failed: {}", e)))?;

        let coll = self.db.collection::<mongodb::bson::Document>("audit_logs");
        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Insert audit log failed: {}", e)))?;

        Ok(())
    }

    async fn list(&self, query: AuditLogQuery) -> AppResult<PaginatedData<AuditLogEntity>> {
        let coll = self.db.collection::<mongodb::bson::Document>("audit_logs");
        let mut filter = doc! {};

        if let Some(admin_id) = query.admin_id {
            filter.insert("admin_id", admin_id.to_string());
        }
        if let Some(ref slug) = query.system_slug {
            filter.insert("system_slug", slug);
        }
        if let Some(ref method) = query.method {
            filter.insert("method", method);
        }

        let total = coll
            .count_documents(filter.clone())
            .await
            .map_err(|e| AppError::Database(format!("Count audit logs failed: {}", e)))?;

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let skip = (page - 1) * page_size;

        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1, "id": -1 })
            .skip(skip)
            .limit(page_size as i64)
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Fetch audit logs failed: {}", e)))?;

        let mut logs = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize audit log failed: {}", e)))?;
            let item: AuditLogEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Audit log mapping error: {}", e)))?;
            logs.push(item);
        }

        Ok(PaginatedData::new(logs, page, page_size, total))
    }
}

// ============================================================================
// MongoApplicationEngine
// ============================================================================

pub struct MongoApplicationEngine {
    db: Database,
}

impl MongoApplicationEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ApplicationStoreEngine for MongoApplicationEngine {
    async fn list_applications(&self, system_slug: &str) -> AppResult<Vec<ApplicationEntity>> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "system_id": system_slug,
            "deleted_at": { "$exists": false }
        };
        let find_opts = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_opts)
            .await
            .map_err(|e| AppError::Database(format!("Failed to list applications: {}", e)))?;

        let mut apps = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Deserialize app failed: {}", e)))?;
            let app: ApplicationEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Application mapping error: {}", e)))?;
            apps.push(app);
        }
        Ok(apps)
    }

    async fn get_application_by_id(&self, id: Uuid) -> AppResult<ApplicationEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "id": id.to_string(),
            "deleted_at": { "$exists": false }
        };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get application failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Application '{}' not found", id)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Application mapping error: {}", e)))
    }

    async fn get_by_key(&self, app_key: &str) -> AppResult<ApplicationEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "app_key": app_key,
            "deleted_at": { "$exists": false }
        };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get application by key failed: {}", e)))?
            .ok_or_else(|| {
                AppError::NotFound(format!("Application with key '{}' not found", app_key))
            })?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Application mapping error: {}", e)))
    }

    async fn find_by_token(&self, token: &str) -> AppResult<Option<ApplicationEntity>> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "$or": [
                { "app_secret": token },
                { "app_key": token }
            ],
            "deleted_at": { "$exists": false }
        };

        let doc = coll.find_one(filter).await.map_err(|e| {
            AppError::Database(format!("Lookup application by token failed: {}", e))
        })?;

        match doc {
            Some(d) => {
                let app: ApplicationEntity = mongodb::bson::from_document(d)
                    .map_err(|e| AppError::Database(format!("Application mapping error: {}", e)))?;
                Ok(Some(app))
            }
            None => Ok(None),
        }
    }

    async fn has_applications(&self, system_slug: &str) -> AppResult<bool> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "system_id": system_slug,
            "deleted_at": { "$exists": false }
        };
        let count = coll
            .count_documents(filter)
            .await
            .map_err(|e| AppError::Database(format!("Count applications failed: {}", e)))?;
        Ok(count > 0)
    }

    async fn create_application(
        &self,
        system_slug: &str,
        name: &str,
        description: Option<&str>,
        permissions: Value,
    ) -> AppResult<ApplicationEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let now = Utc::now();
        let id = Uuid::now_v7();
        let app_key = format!("app_{}", Uuid::now_v7().simple());
        let app_secret = format!("sk_{}", Uuid::now_v7().simple());

        let entity = ApplicationEntity {
            id,
            system_id: system_slug.to_string(),
            name: name.to_string(),
            app_key,
            app_secret,
            description: description.map(|s| s.to_string()),
            status: 1,
            permissions,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Serialize application failed: {}", e)))?;

        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Insert application failed: {}", e)))?;

        Ok(entity)
    }

    async fn update_application(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
        permissions: Option<Value>,
    ) -> AppResult<ApplicationEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "id": id.to_string(),
            "deleted_at": { "$exists": false }
        };

        let mut set_doc = doc! {
            "updated_at": mongodb::bson::to_bson(&Utc::now()).unwrap_or(mongodb::bson::Bson::Null)
        };

        if let Some(n) = name {
            set_doc.insert("name", n);
        }
        if let Some(d) = description {
            set_doc.insert("description", d);
        }
        if let Some(s) = status {
            set_doc.insert("status", s as i32);
        }
        if let Some(p) = permissions {
            let p_bson = mongodb::bson::to_bson(&p)
                .map_err(|e| AppError::Database(format!("Serialize permissions failed: {}", e)))?;
            set_doc.insert("permissions", p_bson);
        }

        let update = doc! { "$set": set_doc };
        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Update application failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!(
                "Application '{}' not found",
                id
            )));
        }

        self.get_application_by_id(id).await
    }

    async fn regenerate_secret(&self, id: Uuid) -> AppResult<ApplicationEntity> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "id": id.to_string(),
            "deleted_at": { "$exists": false }
        };
        let new_secret = format!("sk_{}", Uuid::now_v7().simple());
        let update = doc! {
            "$set": {
                "app_secret": &new_secret,
                "updated_at": mongodb::bson::to_bson(&Utc::now()).unwrap_or(mongodb::bson::Bson::Null)
            }
        };

        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Regenerate secret failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!(
                "Application '{}' not found",
                id
            )));
        }

        self.get_application_by_id(id).await
    }

    async fn delete_application(&self, id: Uuid) -> AppResult<()> {
        let coll = self
            .db
            .collection::<mongodb::bson::Document>("applications");
        let filter = doc! {
            "id": id.to_string(),
            "deleted_at": { "$exists": false }
        };
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::to_bson(&Utc::now()).unwrap_or(mongodb::bson::Bson::Null)
            }
        };

        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Delete application failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!(
                "Application '{}' not found",
                id
            )));
        }

        Ok(())
    }
}

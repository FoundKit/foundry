use crate::entities::ModelRecordEntity;
use crate::engines::mongodb::counters::next_id;
use crate::traits::{RecordQuery, RecordStoreEngine};
use async_trait::async_trait;
use chrono::Utc;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::PaginatedData;
use mongodb::bson::doc;
use mongodb::Database;
use serde_json::Value;

pub struct MongoRecordEngine {
    db: Database,
}

impl MongoRecordEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    async fn resolve_model_id(&self, system_slug: &str, model_slug: &str) -> AppResult<i64> {
        let coll = self.db.collection::<mongodb::bson::Document>("models");
        let filter = doc! {
            "system_id": system_slug,
            "slug": model_slug,
            "deleted_at": null,
        };
        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Failed to resolve model ID: {}", e)))?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Model '{}' not found in system '{}'",
                    model_slug, system_slug
                ))
            })?;

        doc.get_i64("id")
            .map_err(|e| AppError::Database(format!("Invalid model ID in document: {}", e)))
    }
}

#[async_trait]
impl RecordStoreEngine for MongoRecordEngine {
    async fn list(
        &self,
        system_slug: &str,
        model_slug: &str,
        query: RecordQuery,
    ) -> AppResult<PaginatedData<ModelRecordEntity>> {
        let model_id = self.resolve_model_id(system_slug, model_slug).await?;
        let coll = self.db.collection::<mongodb::bson::Document>("model_records");

        let filter = doc! {
            "model_id": model_id,
            "deleted_at": null,
        };

        let total = coll
            .count_documents(filter.clone())
            .await
            .map_err(|e| AppError::Database(format!("Count query failed: {}", e)))?;

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let skip = (page - 1) * page_size;

        let find_options = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1, "id": -1 })
            .skip(skip)
            .limit(page_size as i64)
            .build();

        let mut cursor = coll
            .find(filter)
            .with_options(find_options)
            .await
            .map_err(|e| AppError::Database(format!("Fetch records failed: {}", e)))?;

        let mut records = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| AppError::Database(format!("Cursor error: {}", e)))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| AppError::Database(format!("Failed to deserialize record: {}", e)))?;
            let record: ModelRecordEntity = mongodb::bson::from_document(doc)
                .map_err(|e| AppError::Database(format!("Record mapping error: {}", e)))?;
            records.push(record);
        }

        Ok(PaginatedData::new(records, page, page_size, total))
    }

    async fn get_by_id(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
    ) -> AppResult<ModelRecordEntity> {
        let coll = self.db.collection::<mongodb::bson::Document>("model_records");
        let filter = doc! {
            "id": id,
            "system_id": system_slug,
            "model_slug": model_slug,
            "deleted_at": null,
        };

        let doc = coll
            .find_one(filter)
            .await
            .map_err(|e| AppError::Database(format!("Get record failed: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Record #{} not found", id)))?;

        mongodb::bson::from_document(doc)
            .map_err(|e| AppError::Database(format!("Record mapping error: {}", e)))
    }

    async fn create(
        &self,
        system_slug: &str,
        model_slug: &str,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let model_id = self.resolve_model_id(system_slug, model_slug).await?;
        let new_id = next_id(&self.db, "model_records").await?;
        let now = Utc::now();

        let entity = ModelRecordEntity {
            id: new_id,
            system_id: system_slug.to_string(),
            model_slug: model_slug.to_string(),
            data,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let mut doc = mongodb::bson::to_document(&entity)
            .map_err(|e| AppError::Database(format!("Failed to serialize record: {}", e)))?;
        doc.insert("model_id", model_id);

        let coll = self.db.collection::<mongodb::bson::Document>("model_records");
        coll.insert_one(doc)
            .await
            .map_err(|e| AppError::Database(format!("Failed to insert record: {}", e)))?;

        Ok(entity)
    }

    async fn update(
        &self,
        system_slug: &str,
        model_slug: &str,
        id: i64,
        data: Value,
    ) -> AppResult<ModelRecordEntity> {
        let mut record = self.get_by_id(system_slug, model_slug, id).await?;
        record.data = data;
        record.updated_at = Utc::now();

        let bson_data = mongodb::bson::to_bson(&record.data)
            .map_err(|e| AppError::Database(format!("Failed to serialize data payload: {}", e)))?;

        let coll = self.db.collection::<mongodb::bson::Document>("model_records");
        let filter = doc! {
            "id": id,
            "system_id": system_slug,
            "model_slug": model_slug,
            "deleted_at": null,
        };
        let update = doc! {
            "$set": {
                "data": bson_data,
                "updated_at": mongodb::bson::to_bson(&record.updated_at).unwrap_or(mongodb::bson::Bson::Null),
            }
        };

        coll.update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Update record failed: {}", e)))?;

        Ok(record)
    }

    async fn delete(&self, system_slug: &str, model_slug: &str, id: i64) -> AppResult<()> {
        let coll = self.db.collection::<mongodb::bson::Document>("model_records");
        let filter = doc! {
            "id": id,
            "system_id": system_slug,
            "model_slug": model_slug,
            "deleted_at": null,
        };
        let now = Utc::now();
        let update = doc! {
            "$set": {
                "deleted_at": mongodb::bson::to_bson(&now).unwrap_or(mongodb::bson::Bson::Null),
            }
        };

        let res = coll
            .update_one(filter, update)
            .await
            .map_err(|e| AppError::Database(format!("Delete record failed: {}", e)))?;

        if res.matched_count == 0 {
            return Err(AppError::NotFound(format!("Record #{} not found", id)));
        }

        Ok(())
    }
}

use crate::entities::{PlatformSummary, SystemEntity, SystemItem, SystemStats};
use crate::facade::Database;
pub use crate::traits::SystemQuery;
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;
use uuid::Uuid;

pub struct SystemStore;

impl SystemStore {
    pub async fn list(db: &Database) -> AppResult<Vec<SystemEntity>> {
        db.systems().list().await
    }

    pub async fn list_paginated(
        db: &Database,
        query: SystemQuery,
        allowed_systems: Option<&[String]>,
    ) -> AppResult<PaginatedData<SystemItem>> {
        db.systems().list_paginated(query, allowed_systems).await
    }

    pub async fn get_by_id(db: &Database, id: Uuid) -> AppResult<SystemItem> {
        db.systems().get_by_id(id).await
    }

    pub async fn get_by_slug(db: &Database, slug: &str) -> AppResult<SystemItem> {
        db.systems().get_by_slug(slug).await
    }

    pub async fn get_stats(db: &Database, slug: &str) -> AppResult<SystemStats> {
        db.systems().get_stats(slug).await
    }

    pub async fn platform_summary(db: &Database) -> AppResult<PlatformSummary> {
        db.systems().platform_summary().await
    }

    pub async fn create(
        db: &Database,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<SystemEntity> {
        db.systems().create(slug, name, description).await
    }

    pub async fn update(
        db: &Database,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> AppResult<SystemEntity> {
        db.systems().update(id, name, description, status).await
    }
}

use crate::entities::{PlatformSummary, SystemEntity, SystemItem, SystemStats};
use async_trait::async_trait;
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SystemQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub id: Option<Uuid>,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i16>,
}

#[async_trait]
pub trait SystemStoreEngine: Send + Sync {
    /// Retrieve list of all sub-systems (legacy / simple)
    async fn list(&self) -> AppResult<Vec<SystemEntity>>;

    /// Retrieve paginated list of sub-systems with multi-attribute filtering & live statistics
    async fn list_paginated(
        &self,
        query: SystemQuery,
        allowed_systems: Option<&[String]>,
    ) -> AppResult<PaginatedData<SystemItem>>;

    /// Retrieve single sub-system by UUID
    async fn get_by_id(&self, id: Uuid) -> AppResult<SystemItem>;

    /// Retrieve single sub-system by unique slug
    async fn get_by_slug(&self, slug: &str) -> AppResult<SystemItem>;

    /// Retrieve detailed stats for a sub-system
    async fn get_stats(&self, slug: &str) -> AppResult<SystemStats>;

    /// Retrieve platform-wide summary metrics
    async fn platform_summary(&self) -> AppResult<PlatformSummary>;

    /// Create a new sub-system
    async fn create(
        &self,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<SystemEntity>;

    /// Update an existing sub-system
    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        status: Option<i16>,
    ) -> AppResult<SystemEntity>;
}

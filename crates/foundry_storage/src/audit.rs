use crate::entities::AuditLogEntity;
use crate::facade::Database;
pub use crate::traits::{AuditLogInsert, AuditLogQuery};
use foundry_core::error::AppResult;
use foundry_core::response::PaginatedData;

pub struct AuditStore;

impl AuditStore {
    pub async fn insert(db: &Database, log: AuditLogInsert) -> AppResult<()> {
        db.audit().insert(log).await
    }

    pub async fn list(
        db: &Database,
        query: AuditLogQuery,
    ) -> AppResult<PaginatedData<AuditLogEntity>> {
        db.audit().list(query).await
    }
}

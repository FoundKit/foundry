use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, Query, State},
};
use foundry_auth::AdminClaims;
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::{ApiResponse, PaginatedData};
use foundry_storage::{AuditLogEntity, AuditLogQuery, AuditStore};

pub async fn list_audit_logs_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Query(query): Query<AuditLogQuery>,
) -> AppResult<Json<ApiResponse<PaginatedData<AuditLogEntity>>>> {
    // If not super_admin, constrain query to only allowed systems
    if claims.role != "super_admin" && !claims.allowed_systems.iter().any(|s| s == "*") {
        if let Some(ref slug) = query.system_slug {
            if !claims.allowed_systems.contains(slug) {
                return Err(AppError::Forbidden(
                    "You do not have access to view audit logs for this sub-system".to_string(),
                ));
            }
        } else {
            // Cannot query global logs without super_admin
            return Err(AppError::Forbidden(
                "Please specify a sub-system you are authorized to manage".to_string(),
            ));
        }
    }

    let logs = AuditStore::list(&state.db, query).await?;
    Ok(Json(ApiResponse::success(logs)))
}

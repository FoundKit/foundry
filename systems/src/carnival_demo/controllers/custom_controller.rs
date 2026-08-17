use crate::carnival_demo::dto::{ParticipateRequest, ParticipateResponse};
use crate::carnival_demo::logic::CarnivalService;
use axum::{Json, extract::Extension};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use foundry_core::response::ApiResponse;
use validator::Validate;

pub async fn handle_participate(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<ParticipateRequest>,
) -> AppResult<Json<ApiResponse<ParticipateResponse>>> {
    payload.validate()?;
    let result = CarnivalService::participate(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}

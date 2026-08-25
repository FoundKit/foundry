use axum::{Json, Router, extract::Extension, routing::post};
use foundry::prelude::*;
use validator::Validate;

use crate::systems::newsletter::dto::{SubscribeRequest, SubscriberResponse};
use crate::systems::newsletter::logic::NewsletterService;

pub fn build_routes() -> Router {
    Router::new().route("/subscribe", post(handle_subscribe))
}

pub async fn handle_subscribe(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<SubscribeRequest>,
) -> AppResult<Json<ApiResponse<SubscriberResponse>>> {
    payload.validate()?;
    let result = NewsletterService::subscribe(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(result)))
}

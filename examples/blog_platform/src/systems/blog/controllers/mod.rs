use axum::{Json, Router, extract::Extension, routing::post};
use foundry::prelude::*;
use validator::Validate;

use crate::systems::blog::dto::{CreatePostRequest, PostResponse, PublishPostRequest};
use crate::systems::blog::logic::BlogService;

pub fn build_routes() -> Router {
    Router::new()
        .route("/posts", post(handle_create_post))
        .route("/posts/publish", post(handle_publish_post))
}

pub async fn handle_create_post(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<CreatePostRequest>,
) -> AppResult<Json<ApiResponse<PostResponse>>> {
    payload.validate()?;
    let post = BlogService::create_post(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(post)))
}

pub async fn handle_publish_post(
    Extension(ctx): Extension<SystemContext>,
    Json(payload): Json<PublishPostRequest>,
) -> AppResult<Json<ApiResponse<PostResponse>>> {
    payload.validate()?;
    let post = BlogService::publish_post(&ctx, payload).await?;
    Ok(Json(ApiResponse::success(post)))
}

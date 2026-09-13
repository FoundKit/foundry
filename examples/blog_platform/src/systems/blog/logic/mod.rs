use crate::systems::blog::dto::{CreatePostRequest, PostResponse, PublishPostRequest};
use chrono::Utc;
use foundry::prelude::*;

pub struct BlogService;

impl BlogService {
    pub async fn create_post(
        _ctx: &SystemContext,
        req: CreatePostRequest,
    ) -> AppResult<PostResponse> {
        // Business logic: Generate new post record
        let post = PostResponse {
            id: 1001,
            title: req.title,
            content: req.content,
            author: req.author,
            tags: req.tags,
            published: false,
            created_at: Utc::now(),
        };

        Ok(post)
    }

    pub async fn publish_post(
        _ctx: &SystemContext,
        req: PublishPostRequest,
    ) -> AppResult<PostResponse> {
        let post = PostResponse {
            id: req.post_id,
            title: "Published Article: Rust 2026".to_string(),
            content: "Foundry framework architecture walkthrough".to_string(),
            author: "dev@example.com".to_string(),
            tags: vec!["rust".to_string(), "framework".to_string()],
            published: true,
            created_at: Utc::now(),
        };

        Ok(post)
    }
}

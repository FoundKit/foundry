use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 3, max = 120, message = "Title must be between 3 and 120 chars"))]
    pub title: String,
    #[validate(length(min = 5, message = "Content must be at least 5 chars"))]
    pub content: String,
    pub author: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PublishPostRequest {
    pub post_id: u64,
    pub notify_subscribers: bool,
}

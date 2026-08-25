use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubscribeRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberResponse {
    pub id: u64,
    pub email: String,
    pub active: bool,
    pub subscribed_at: DateTime<Utc>,
}

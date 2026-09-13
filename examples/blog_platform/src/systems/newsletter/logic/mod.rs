use crate::systems::newsletter::dto::{SubscribeRequest, SubscriberResponse};
use chrono::Utc;
use foundry::prelude::*;

pub struct NewsletterService;

impl NewsletterService {
    pub async fn subscribe(
        _ctx: &SystemContext,
        req: SubscribeRequest,
    ) -> AppResult<SubscriberResponse> {
        Ok(SubscriberResponse {
            id: 501,
            email: req.email,
            active: true,
            subscribed_at: Utc::now(),
        })
    }
}

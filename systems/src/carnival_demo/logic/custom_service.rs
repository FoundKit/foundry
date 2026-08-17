use crate::carnival_demo::dto::{ParticipateRequest, ParticipateResponse};
use foundry_core::context::SystemContext;
use foundry_core::error::AppResult;
use uuid::Uuid;

pub struct CarnivalService;

impl CarnivalService {
    pub async fn participate(
        _ctx: &SystemContext,
        payload: ParticipateRequest,
    ) -> AppResult<ParticipateResponse> {
        let ticket_id = format!("TKT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let is_winner = payload.lucky_number.is_multiple_of(7);
        let prize_name = if is_winner {
            Some("Golden Carnival Gift Box".to_string())
        } else {
            None
        };

        Ok(ParticipateResponse {
            ticket_id,
            nickname: payload.nickname,
            lucky_number: payload.lucky_number,
            is_winner,
            prize_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_carnival_participate_winner() {
        let ctx = SystemContext::new("carnival_demo");
        let req = ParticipateRequest {
            nickname: "Bob".to_string(),
            lucky_number: 14, // multiple of 7
        };

        let res = CarnivalService::participate(&ctx, req).await.unwrap();
        assert_eq!(res.nickname, "Bob");
        assert_eq!(res.lucky_number, 14);
        assert!(res.is_winner);
        assert_eq!(res.prize_name, Some("Golden Carnival Gift Box".to_string()));
        assert!(res.ticket_id.starts_with("TKT-"));
    }

    #[tokio::test]
    async fn test_carnival_participate_not_winner() {
        let ctx = SystemContext::new("carnival_demo");
        let req = ParticipateRequest {
            nickname: "Charlie".to_string(),
            lucky_number: 10, // not multiple of 7
        };

        let res = CarnivalService::participate(&ctx, req).await.unwrap();
        assert_eq!(res.nickname, "Charlie");
        assert_eq!(res.lucky_number, 10);
        assert!(!res.is_winner);
        assert_eq!(res.prize_name, None);
    }
}

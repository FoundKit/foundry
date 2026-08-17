use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ParticipateRequest {
    #[validate(length(
        min = 2,
        max = 32,
        message = "Nickname must be between 2 and 32 characters"
    ))]
    pub nickname: String,
    #[validate(range(min = 1, max = 100, message = "Lucky number must be between 1 and 100"))]
    pub lucky_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipateResponse {
    pub ticket_id: String,
    pub nickname: String,
    pub lucky_number: u32,
    pub is_winner: bool,
    pub prize_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_participate_request_validation() {
        let valid = ParticipateRequest {
            nickname: "Alice".to_string(),
            lucky_number: 42,
        };
        assert!(valid.validate().is_ok());

        let invalid_nick = ParticipateRequest {
            nickname: "A".to_string(),
            lucky_number: 42,
        };
        assert!(invalid_nick.validate().is_err());

        let invalid_num = ParticipateRequest {
            nickname: "Alice".to_string(),
            lucky_number: 105,
        };
        assert!(invalid_num.validate().is_err());
    }
}

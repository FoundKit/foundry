use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard localized error envelope according to Foundry architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: u32,
    pub message: String,
    pub i18n_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Database Error: {0}")]
    Database(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> u32 {
        match self {
            Self::BadRequest(_) => 40000,
            Self::Validation(_) => 42200,
            Self::Unauthorized(_) => 40100,
            Self::Forbidden(_) => 40300,
            Self::NotFound(_) => 40400,
            Self::Conflict(_) => 40900,
            Self::Internal(_) => 50000,
            Self::Database(_) => 50001,
        }
    }

    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "errors.bad_request",
            Self::Validation(_) => "errors.validation_failed",
            Self::Unauthorized(_) => "errors.unauthorized",
            Self::Forbidden(_) => "errors.forbidden",
            Self::NotFound(_) => "errors.not_found",
            Self::Conflict(_) => "errors.conflict",
            Self::Internal(_) => "errors.internal_server_error",
            Self::Database(_) => "errors.database_error",
        }
    }

    pub fn envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.error_code(),
            message: self.to_string(),
            i18n_key: self.i18n_key().to_string(),
            args: None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.envelope());
        (status, body).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON parsing error: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_and_envelopes() {
        let err1 = AppError::BadRequest("bad payload".to_string());
        assert_eq!(err1.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err1.error_code(), 40000);
        assert_eq!(err1.i18n_key(), "errors.bad_request");

        let env = err1.envelope();
        assert_eq!(env.code, 40000);
        assert_eq!(env.i18n_key, "errors.bad_request");
        assert!(env.message.contains("bad payload"));

        let err2 = AppError::NotFound("item not found".to_string());
        assert_eq!(err2.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err2.error_code(), 40400);

        let err3 = AppError::Validation("invalid field".to_string());
        assert_eq!(err3.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

        let err4 = AppError::Unauthorized("auth required".to_string());
        assert_eq!(err4.status_code(), StatusCode::UNAUTHORIZED);

        let err5 = AppError::Forbidden("no permission".to_string());
        assert_eq!(err5.status_code(), StatusCode::FORBIDDEN);
    }
}

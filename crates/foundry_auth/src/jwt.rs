use chrono::{Duration, Utc};
use foundry_core::error::{AppError, AppResult};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: Uuid, // admin_id
    pub username: String,
    pub role: String, // "super_admin" | "admin" | "topic_admin"
    pub allowed_systems: Vec<String>,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at
}

impl AdminClaims {
    pub fn is_super_admin(&self) -> bool {
        self.role == "super_admin"
    }

    pub fn is_general_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_topic_admin(&self) -> bool {
        self.role == "topic_admin"
    }

    /// Super Admin and General Admin have full platform-wide sub-system management rights
    pub fn has_platform_manage_access(&self) -> bool {
        self.role == "super_admin"
            || self.role == "admin"
            || self.allowed_systems.iter().any(|s| s == "*")
    }

    /// Only Super Admin can view and manage administrators
    pub fn can_manage_admins(&self) -> bool {
        self.role == "super_admin"
    }

    /// Super Admin and General Admin can view platform-wide summary statistics
    pub fn can_view_platform_summary(&self) -> bool {
        self.role == "super_admin" || self.role == "admin"
    }
}

pub struct JwtService {
    secret: String,
    expire_hours: i64,
}

impl JwtService {
    pub fn new(secret: impl Into<String>, expire_hours: i64) -> Self {
        Self {
            secret: secret.into(),
            expire_hours,
        }
    }

    pub fn generate_token(
        &self,
        admin_id: Uuid,
        username: &str,
        role: &str,
        allowed_systems: Vec<String>,
    ) -> AppResult<String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(self.expire_hours)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = AdminClaims {
            sub: admin_id,
            username: username.to_string(),
            role: role.to_string(),
            allowed_systems,
            exp,
            iat,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate JWT: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> AppResult<AdminClaims> {
        let validation = Validation::default();
        let token_data = decode::<AdminClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {}", e)))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let jwt = JwtService::new("secret_test_key_1234567890", 24);
        let admin_id = Uuid::new_v4();
        let token = jwt
            .generate_token(admin_id, "admin", "super_admin", vec!["*".to_string()])
            .unwrap();

        let claims = jwt.verify_token(&token).unwrap();
        assert_eq!(claims.sub, admin_id);
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, "super_admin");
        assert_eq!(claims.allowed_systems, vec!["*"]);
    }
}

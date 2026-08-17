use crate::jwt::AdminClaims;
use foundry_core::error::{AppError, AppResult};

/// Check if the administrator is permitted to access/modify a target sub-system
pub fn check_system_access(claims: &AdminClaims, target_system_slug: &str) -> AppResult<()> {
    if claims.role == "super_admin" {
        return Ok(());
    }

    if claims
        .allowed_systems
        .iter()
        .any(|s| s == "*" || s == target_system_slug)
    {
        return Ok(());
    }

    Err(AppError::Forbidden(format!(
        "Administrator '{}' is not authorized to manage sub-system '{}'",
        claims.username, target_system_slug
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_super_admin_access() {
        let claims = AdminClaims {
            sub: Uuid::new_v4(),
            username: "admin".to_string(),
            role: "super_admin".to_string(),
            allowed_systems: vec!["*".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };

        assert!(check_system_access(&claims, "carnival_2026").is_ok());
        assert!(check_system_access(&claims, "vip_mall").is_ok());
    }

    #[test]
    fn test_normal_admin_access() {
        let claims = AdminClaims {
            sub: Uuid::new_v4(),
            username: "operator".to_string(),
            role: "admin".to_string(),
            allowed_systems: vec!["carnival_2026".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };

        assert!(check_system_access(&claims, "carnival_2026").is_ok());
        assert!(check_system_access(&claims, "vip_mall").is_err());
    }
}

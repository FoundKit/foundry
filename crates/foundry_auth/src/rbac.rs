use crate::jwt::AdminClaims;
use foundry_core::error::{AppError, AppResult};

/// Check if the administrator is permitted to access/modify a target sub-system
pub fn check_system_access(claims: &AdminClaims, target_system_slug: &str) -> AppResult<()> {
    // Super Admin and General Admin have access across all sub-systems
    if claims.role == "super_admin" || claims.role == "admin" {
        return Ok(());
    }

    // Topic Admin can only access specifically assigned sub-systems
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
            username: "super_boss".to_string(),
            role: "super_admin".to_string(),
            allowed_systems: vec!["*".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };

        assert!(claims.is_super_admin());
        assert!(claims.can_manage_admins());
        assert!(claims.has_platform_manage_access());
        assert!(claims.can_view_platform_summary());
        assert!(check_system_access(&claims, "carnival_2026").is_ok());
        assert!(check_system_access(&claims, "vip_mall").is_ok());
    }

    #[test]
    fn test_general_admin_access() {
        let claims = AdminClaims {
            sub: Uuid::new_v4(),
            username: "general_op".to_string(),
            role: "admin".to_string(),
            allowed_systems: vec!["*".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };

        assert!(claims.is_general_admin());
        assert!(!claims.can_manage_admins());
        assert!(claims.has_platform_manage_access());
        assert!(claims.can_view_platform_summary());
        assert!(check_system_access(&claims, "carnival_2026").is_ok());
        assert!(check_system_access(&claims, "vip_mall").is_ok());
    }

    #[test]
    fn test_topic_admin_access() {
        let claims = AdminClaims {
            sub: Uuid::new_v4(),
            username: "carnival_manager".to_string(),
            role: "topic_admin".to_string(),
            allowed_systems: vec!["carnival_2026".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };

        assert!(claims.is_topic_admin());
        assert!(!claims.can_manage_admins());
        assert!(!claims.can_view_platform_summary());
        assert!(check_system_access(&claims, "carnival_2026").is_ok());
        assert!(check_system_access(&claims, "vip_mall").is_err());
    }
}

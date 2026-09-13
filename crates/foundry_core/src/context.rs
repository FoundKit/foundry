use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Execution context for tenant sub-system operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Internal immutable UUID for the sub-system
    pub system_id: Option<Uuid>,
    /// Unique URL/code slug (e.g., "carnival_2026", "vip_mall")
    pub system_slug: String,
    /// Display name of the sub-system
    pub system_name: String,
    /// Requested locale parsed from Accept-Language / query
    pub locale: String,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Client User-Agent
    pub user_agent: Option<String>,
}

impl SystemContext {
    pub fn new(slug: impl Into<String>) -> Self {
        let slug_str = slug.into();
        Self {
            system_id: None,
            system_slug: slug_str.clone(),
            system_name: slug_str,
            locale: "en-US".to_string(),
            client_ip: None,
            user_agent: None,
        }
    }

    pub fn with_details(
        system_id: Option<Uuid>,
        system_slug: impl Into<String>,
        system_name: impl Into<String>,
        locale: impl Into<String>,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            system_id,
            system_slug: system_slug.into(),
            system_name: system_name.into(),
            locale: locale.into(),
            client_ip,
            user_agent,
        }
    }

    /// Redis key helper scoped to this tenant sub-system
    pub fn redis_key(&self, key: &str) -> String {
        format!("foundry:{}:{}", self.system_slug, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_context() {
        let ctx = SystemContext::new("carnival_2026");
        assert_eq!(ctx.system_slug, "carnival_2026");
        assert_eq!(ctx.system_name, "carnival_2026");
        assert_eq!(ctx.locale, "en-US");
        assert_eq!(
            ctx.redis_key("session:123"),
            "foundry:carnival_2026:session:123"
        );

        let id = Uuid::new_v4();
        let detailed = SystemContext::with_details(
            Some(id),
            "vip_mall",
            "VIP Mall",
            "zh-CN",
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );
        assert_eq!(detailed.system_id, Some(id));
        assert_eq!(detailed.system_slug, "vip_mall");
        assert_eq!(detailed.system_name, "VIP Mall");
        assert_eq!(detailed.locale, "zh-CN");
        assert_eq!(detailed.client_ip.as_deref(), Some("192.168.1.1"));
    }
}

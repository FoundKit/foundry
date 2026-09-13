use axum::Router;
use serde::{Deserialize, Serialize};

/// Specification for custom subsystem admin UI extension pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAdminPageSpec {
    pub key: String,
    pub title: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub entry: String,
    pub required_role: Option<String>,
}

/// Standard Trait for code-first sub-system modules in `systems/src/*`
pub trait SubsystemModule: Send + Sync + 'static {
    /// Returns the unique slug of the sub-system (e.g. "carnival_2026")
    fn slug(&self) -> &'static str;

    /// Human-readable display name of the sub-system
    fn display_name(&self) -> &'static str;

    /// Mounts custom HTTP routes under `/custom-api/v1/s/{slug}/...`
    fn register_routes(&self, router: Router) -> Router;

    /// Optional description of the sub-system
    fn description(&self) -> &'static str {
        ""
    }

    /// Custom admin UI pages registered by this sub-system module
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![]
    }
}

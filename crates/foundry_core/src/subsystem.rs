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

    /// Mounts custom HTTP routes under `/api/v1/s/{slug}/ext/...`
    fn register_routes(&self, router: Router) -> Router {
        router
    }

    /// Optional description of the sub-system
    fn description(&self) -> &'static str {
        ""
    }

    /// Returns the local directory path for custom admin UI pages, if any.
    /// By default, automatically discovers `src/systems/{slug}/custom_pages` or `static/custom_pages/{slug}`.
    fn custom_pages_dir(&self) -> Option<std::path::PathBuf> {
        let slug = self.slug();
        let possible_dirs = [
            std::path::PathBuf::from(format!("src/systems/{}/custom_pages", slug)),
            std::path::PathBuf::from(format!(
                "examples/blog_platform/src/systems/{}/custom_pages",
                slug
            )),
            std::path::PathBuf::from(format!("static/custom_pages/{}", slug)),
        ];
        possible_dirs.into_iter().find(|dir| dir.exists())
    }

    /// Custom admin UI pages registered by this sub-system module
    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![]
    }

    /// Code-defined ORM schemas for this subsystem module.
    /// Relational databases (PostgreSQL, MySQL, MariaDB) will automatically compile
    /// these schemas into DDL migrations and tables.
    fn orm_models(&self) -> Vec<crate::orm::EntitySchema> {
        vec![]
    }

    /// Code-defined custom DDL migration scripts for this subsystem module.
    /// Returns dialect-specific DDL/SQL statements for "postgres", "mysql", "mariadb", "mongodb", or "*"
    fn migration_ddl(&self, _db_type: &str) -> Option<String> {
        None
    }
}

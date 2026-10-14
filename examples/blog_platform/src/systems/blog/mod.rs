pub mod controllers;
pub mod dto;
pub mod logic;

use foundry::prelude::*;

pub struct BlogSubsystem;

impl SubsystemModule for BlogSubsystem {
    fn slug(&self) -> &'static str {
        "blog"
    }

    fn display_name(&self) -> &'static str {
        "Content & Blog Platform"
    }

    fn description(&self) -> &'static str {
        "Articles, publishing workflow, and content management"
    }

    fn register_routes(&self, router: Router) -> Router {
        router.merge(controllers::build_routes())
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "article_editor".to_string(),
            title: "Article Studio".to_string(),
            icon: "FileEdit".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/blog/ext/custom-pages/article_editor.html".to_string(),
            required_role: None,
        }]
    }
}

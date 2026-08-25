pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;
use std::path::PathBuf;
use tower_http::services::ServeDir;

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
        let mut r = router.merge(controllers::build_routes());

        // Serve custom admin UI pages
        let possible_dirs = [
            PathBuf::from("examples/blog_platform/src/systems/blog/custom_pages"),
            PathBuf::from("src/systems/blog/custom_pages"),
            PathBuf::from("static/custom_pages/blog"),
        ];

        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }

        r
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

pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry::prelude::*;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub struct NewsletterSubsystem;

impl SubsystemModule for NewsletterSubsystem {
    fn slug(&self) -> &'static str {
        "newsletter"
    }

    fn display_name(&self) -> &'static str {
        "Newsletter & Email Marketing"
    }

    fn description(&self) -> &'static str {
        "Audience management, subscriber opt-ins, and broadcast campaigns"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());

        let possible_dirs = [
            PathBuf::from("examples/blog_platform/src/systems/newsletter/custom_pages"),
            PathBuf::from("src/systems/newsletter/custom_pages"),
            PathBuf::from("static/custom_pages/newsletter"),
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
            key: "subscribers".to_string(),
            title: "Audience Dashboard".to_string(),
            icon: "Users".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/newsletter/ext/custom-pages/subscribers.html".to_string(),
            required_role: None,
        }]
    }
}

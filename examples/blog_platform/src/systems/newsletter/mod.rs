pub mod controllers;
pub mod dto;
pub mod logic;

use foundry::prelude::*;

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
        router.merge(controllers::build_routes())
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

pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry_core::SubsystemModule;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub struct CarnivalDemoModule;

impl SubsystemModule for CarnivalDemoModule {
    fn slug(&self) -> &'static str {
        "carnival_demo"
    }

    fn display_name(&self) -> &'static str {
        "Carnival 2026 Demo Subsystem"
    }

    fn description(&self) -> &'static str {
        "Demo sub-system showcasing custom 3-layer architecture, DTO validation, and domain services"
    }

    fn register_routes(&self, router: Router) -> Router {
        let mut r = router.merge(controllers::build_routes());
        let possible_dirs = [
            PathBuf::from("systems/src/carnival_demo/custom_pages"),
            PathBuf::from("systems/carnival_demo/custom_pages"),
            PathBuf::from("../systems/src/carnival_demo/custom_pages"),
            PathBuf::from("static/custom_pages/carnival_demo"),
        ];
        for dir in possible_dirs {
            if dir.exists() {
                r = r.nest_service("/custom-pages", ServeDir::new(dir));
                break;
            }
        }
        r
    }

    fn custom_admin_pages(&self) -> Vec<foundry_core::CustomAdminPageSpec> {
        vec![
            foundry_core::CustomAdminPageSpec {
                key: "lottery_dashboard".to_string(),
                title: "抽奖运营大屏".to_string(),
                icon: "Gift".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/carnival_demo/ext/custom-pages/lottery_dashboard.html"
                    .to_string(),
                required_role: None,
            },
            foundry_core::CustomAdminPageSpec {
                key: "wheel_control".to_string(),
                title: "转盘概率调控".to_string(),
                icon: "Sparkles".to_string(),
                page_type: "iframe".to_string(),
                entry: "/api/v1/s/carnival_demo/ext/custom-pages/wheel_control.html".to_string(),
                required_role: Some("super_admin".to_string()),
            },
        ]
    }
}

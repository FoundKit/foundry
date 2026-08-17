pub mod controllers;
pub mod dto;
pub mod logic;

use axum::Router;
use foundry_core::SubsystemModule;

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
        router.merge(controllers::build_routes())
    }
}

use axum::Router;

/// Standard Trait for code-first sub-system modules in `systems/src/*`
pub trait SubsystemModule: Send + Sync + 'static {
    /// Returns the unique slug of the sub-system (e.g. "carnival_2026")
    fn slug(&self) -> &'static str;

    /// Human-readable display name of the sub-system
    fn display_name(&self) -> &'static str;

    /// Mounts custom HTTP routes under `/api/v1/s/{slug}/...`
    fn register_routes(&self, router: Router) -> Router;

    /// Optional description of the sub-system
    fn description(&self) -> &'static str {
        ""
    }
}

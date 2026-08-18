use crate::handlers::*;
use crate::middleware::{
    audit_interceptor, extract_system_context, require_admin_auth, require_topic_access,
};
use crate::state::AppState;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post, put},
};
use foundry_core::SubsystemModule;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_router(state: AppState, subsystems: Vec<Box<dyn SubsystemModule>>) -> Router {
    // 1. Public endpoints
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/admin/auth/login", post(auth::login_handler));

    // 2. Admin Control Plane routes (requires admin auth)
    let admin_routes = Router::new()
        .route("/admin/auth/me", get(auth::me_handler))
        .route(
            "/admin/platform/summary",
            get(systems::get_platform_summary_handler),
        )
        .route(
            "/admin/systems",
            get(systems::list_systems_handler).post(systems::create_system_handler),
        )
        .route(
            "/admin/systems/{id}",
            get(systems::get_system_handler).put(systems::update_system_handler),
        )
        .route(
            "/admin/admins",
            get(admins::list_admins_handler).post(admins::create_admin_handler),
        )
        .route("/admin/admins/{id}", put(admins::update_admin_handler))
        .route("/admin/audit-logs", get(audit::list_audit_logs_handler))
        // Topic-scoped admin routes
        .route(
            "/admin/s/{system_slug}/details",
            get(systems::get_system_by_slug_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/admin/s/{system_slug}/stats",
            get(systems::get_system_stats_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/admin/s/{system_slug}/configs/schema",
            get(configs::list_config_schema_handler)
                .post(configs::upsert_config_schema_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/admin/s/{system_slug}/models",
            get(models::list_models_handler)
                .post(models::create_model_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/admin/s/{system_slug}/models/{id}/fields",
            get(models::list_model_fields_handler)
                .post(models::add_model_field_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_admin_auth));

    // 3. Sub-system dynamic REST API routes (/api/v1/s/{system_slug}/...)
    let dynamic_routes = Router::new()
        .route(
            "/s/{system_slug}/configs",
            get(configs::get_aggregated_configs_handler)
                .put(configs::update_aggregated_configs_handler),
        )
        .route(
            "/s/{system_slug}/{model_slug}",
            get(autocrud::list_records_handler).post(autocrud::create_record_handler),
        )
        .route(
            "/s/{system_slug}/{model_slug}/{id}",
            get(autocrud::get_record_handler)
                .put(autocrud::update_record_handler)
                .patch(autocrud::update_record_handler)
                .delete(autocrud::delete_record_handler),
        );

    // 4. Custom code-first sub-system routes
    let mut custom_subsystems_router = Router::new();
    for sub in subsystems {
        tracing::info!(
            "Registering custom sub-system: {} ({})",
            sub.display_name(),
            sub.slug()
        );
        let sub_router = sub.register_routes(Router::new());
        custom_subsystems_router =
            custom_subsystems_router.nest_service(&format!("/s/{}", sub.slug()), sub_router);
    }

    // Merge into /api/v1
    let api_v1 = Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .merge(custom_subsystems_router)
        .merge(dynamic_routes);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(from_fn_with_state(state.clone(), audit_interceptor))
        .layer(from_fn_with_state(state.clone(), extract_system_context))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

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
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;


pub fn build_router(state: AppState) -> Router {
    // 1. Admin Control Plane APIs (/api/v1/admin/*)
    let admin_public_routes = Router::new().route("/auth/login", post(auth::login_handler));

    let admin_protected_routes = Router::new()
        .route("/auth/me", get(auth::me_handler))
        .route("/platform/summary", get(systems::get_platform_summary_handler))
        .route(
            "/systems",
            get(systems::list_systems_handler).post(systems::create_system_handler),
        )
        .route(
            "/systems/{id}",
            get(systems::get_system_handler).put(systems::update_system_handler),
        )
        .route(
            "/admins",
            get(admins::list_admins_handler).post(admins::create_admin_handler),
        )
        .route("/admins/{id}", put(admins::update_admin_handler))
        .route("/audit-logs", get(audit::list_audit_logs_handler))
        // Topic-scoped admin routes
        .route(
            "/s/{system_slug}/details",
            get(systems::get_system_by_slug_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/stats",
            get(systems::get_system_stats_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/configs/schema",
            get(configs::list_config_schema_handler)
                .post(configs::upsert_config_schema_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models",
            get(models::list_models_handler)
                .post(models::create_model_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models/{id}/fields",
            get(models::list_model_fields_handler)
                .post(models::add_model_field_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/custom-pages",
            get(systems::list_subsystem_custom_pages_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_admin_auth));

    let admin_api = Router::new()
        .merge(admin_public_routes)
        .merge(admin_protected_routes);

    // 2. Auto-generated Dynamic REST CRUD APIs (/api/v1/s/{system_slug}/{model_slug})
    let autocrud_api = Router::new()
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

    // 3. Custom Sub-system Extension APIs (/api/v1/s/{system_slug}/ext/*)
    let mut custom_ext_api = Router::new();
    for sub in state.subsystems.iter() {
        tracing::info!(
            "Mounting custom subsystem extension API: {} ({}) under /api/v1/s/{}/ext",
            sub.display_name(),
            sub.slug(),
            sub.slug()
        );
        let sub_router = sub.register_routes(Router::new());
        custom_ext_api =
            custom_ext_api.nest_service(&format!("/s/{}/ext", sub.slug()), sub_router);
    }

    // Combine into uniform RESTful API tree (/api/v1)
    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .nest("/admin", admin_api)
        .merge(autocrud_api)
        .merge(custom_ext_api);

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

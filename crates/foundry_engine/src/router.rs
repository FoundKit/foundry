use crate::handlers::*;
use crate::middleware::{
    audit_interceptor, extract_system_context, require_admin_auth,
    require_subsystem_app_or_admin_auth, require_topic_access,
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
    // 1. Admin Control Plane APIs (/api/admin/*)
    let admin_public_routes = Router::new()
        .route("/auth/settings", get(auth::public_auth_settings_handler))
        .route("/auth/login", post(auth::login_handler))
        .route("/auth/email/send-code", post(auth::send_login_code_handler))
        .route("/auth/email/login", post(auth::email_login_handler));

    let mut admin_protected_routes = Router::new()
        .route("/auth/me", get(auth::me_handler))
        .route("/auth/password", put(auth::change_password_handler))
        .route(
            "/auth/bind-email/send-code",
            post(auth::send_bind_email_code_handler),
        )
        .route(
            "/auth/bind-email/verify",
            post(auth::verify_bind_email_handler),
        )
        .route(
            "/platform/login-settings",
            get(auth::get_login_settings_handler).put(auth::update_login_settings_handler),
        )
        .route(
            "/platform/summary",
            get(systems::get_platform_summary_handler),
        )
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
            "/s/{system_slug}/models/trash",
            get(models::list_deleted_models_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models/{id}",
            put(models::update_model_handler)
                .delete(models::delete_model_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models/{id}/restore",
            post(models::restore_model_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models/{id}/fields",
            get(models::list_model_fields_handler)
                .post(models::add_model_field_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/models/{id}/fields/{field_id}",
            put(models::update_model_field_handler)
                .delete(models::delete_model_field_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/custom-pages",
            get(systems::list_subsystem_custom_pages_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        // Subsystem Client Applications Management
        .route(
            "/s/{system_slug}/apps",
            get(apps::list_applications_handler)
                .post(apps::create_application_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/apps/{id}",
            get(apps::get_application_handler)
                .put(apps::update_application_handler)
                .delete(apps::delete_application_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route(
            "/s/{system_slug}/apps/{id}/regenerate-secret",
            post(apps::regenerate_app_secret_handler)
                .route_layer(from_fn_with_state(state.clone(), require_topic_access)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_admin_auth));

    // Subsystem Admin Management Extension APIs (/api/admin/ext/{slug}/*)
    for sub in state.subsystems.iter() {
        tracing::info!(
            "Mounting custom subsystem admin API: {} ({}) under /api/admin/ext/{}",
            sub.display_name(),
            sub.slug(),
            sub.slug()
        );
        let sub_admin_router = sub
            .register_admin_routes(Router::new())
            .layer(from_fn_with_state(state.clone(), require_topic_access))
            .layer(from_fn_with_state(state.clone(), require_admin_auth));
        admin_protected_routes =
            admin_protected_routes.nest_service(&format!("/ext/{}", sub.slug()), sub_admin_router);
    }

    let admin_api = Router::new()
        .merge(admin_public_routes)
        .merge(admin_protected_routes);

    // 2. Auto-generated Dynamic REST CRUD APIs (/api/s/{system_slug}/{model_slug})
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
        )
        .route_layer(from_fn_with_state(
            state.clone(),
            require_subsystem_app_or_admin_auth,
        ));

    // 3. Custom Sub-system Public Extension APIs (/api/ext/{system_slug}/*)
    let mut custom_ext_public_api = Router::new();
    for sub in state.subsystems.iter() {
        tracing::info!(
            "Mounting custom subsystem public extension API: {} ({}) under /api/ext/{}",
            sub.display_name(),
            sub.slug(),
            sub.slug()
        );
        let sub_router = sub.register_routes(Router::new());
        custom_ext_public_api =
            custom_ext_public_api.nest_service(&format!("/{}", sub.slug()), sub_router);
    }

    // Combine into uniform RESTful API tree (/api)
    let api = Router::new()
        .route("/health", get(health_check))
        .nest("/admin", admin_api)
        .merge(autocrud_api)
        .nest_service("/ext", custom_ext_public_api);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .nest("/api", api)
        .layer(from_fn_with_state(state.clone(), audit_interceptor))
        .layer(from_fn_with_state(state.clone(), extract_system_context))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Root redirect to Admin UI (/ -> /admin/)
    router = router
        .route(
            "/",
            get(|| async { axum::response::Redirect::temporary("/admin/") }),
        )
        .route("/health", get(health_check));

    // 4. Subsystem Custom Admin Pages (/admin/ext/{slug}/*)
    let mut ext_pages_router = Router::new();
    for sub in state.subsystems.iter() {
        if let Some(pages_dir) = sub.custom_pages_dir() {
            tracing::info!(
                "Mounting custom subsystem admin pages for slug '{}' from {:?}",
                sub.slug(),
                pages_dir
            );
            let serve_dir = tower_http::services::ServeDir::new(pages_dir)
                .append_index_html_on_directories(true);
            ext_pages_router =
                ext_pages_router.nest_service(&format!("/{}", sub.slug()), serve_dir);
        }
    }

    let mut admin_ui_router = Router::new().nest_service("/ext", ext_pages_router);

    // If static Admin SPA build exists on disk, serve it under /admin; otherwise fall back to embedded SPA
    let admin_static_dirs = [
        std::path::PathBuf::from("static/admin"),
        std::path::PathBuf::from("apps/admin/dist"),
        std::path::PathBuf::from("../apps/admin/dist"),
    ];

    let mut mounted_disk_admin = false;
    for dir in admin_static_dirs {
        if dir.exists() {
            let index_file = dir.join("index.html");
            if index_file.exists() {
                let serve_dir = tower_http::services::ServeDir::new(dir.clone())
                    .append_index_html_on_directories(true)
                    .not_found_service(tower_http::services::ServeFile::new(index_file));
                admin_ui_router = admin_ui_router.fallback_service(serve_dir);
                mounted_disk_admin = true;
                break;
            }
        }
    }

    if !mounted_disk_admin {
        tracing::info!("Mounting embedded Admin UI SPA assets under /admin");
        admin_ui_router = admin_ui_router.fallback(admin_embedded_handler);
    }

    router = router.nest_service("/admin", admin_ui_router);

    // Also mount /assets fallback for direct asset requests
    router = router.route("/assets/{*path}", get(assets_embedded_handler));

    router
}

#[derive(rust_embed::Embed)]
#[folder = "admin_dist"]
struct EmbeddedAdminAssets;

async fn serve_embedded_file(path: &str) -> axum::response::Response {
    use axum::http::{HeaderValue, StatusCode, header};
    use axum::response::IntoResponse;

    let clean_path = path.trim_start_matches('/');
    let target = if clean_path.is_empty() {
        "index.html"
    } else {
        clean_path
    };

    if let Some(file) = EmbeddedAdminAssets::get(target) {
        let mime = mime_guess::from_path(target).first_or_octet_stream();
        let content_type = HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
        return ([(header::CONTENT_TYPE, content_type)], file.data).into_response();
    }

    // SPA client-side fallback: return index.html
    if let Some(index) = EmbeddedAdminAssets::get("index.html") {
        return (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )],
            index.data,
        )
            .into_response();
    }

    (StatusCode::NOT_FOUND, "Admin UI assets not found").into_response()
}

async fn admin_embedded_handler(uri: axum::http::Uri) -> axum::response::Response {
    let path = uri.path();
    let relative = path.strip_prefix("/admin").unwrap_or(path);
    serve_embedded_file(relative).await
}

async fn assets_embedded_handler(uri: axum::http::Uri) -> axum::response::Response {
    serve_embedded_file(uri.path()).await
}

async fn health_check() -> &'static str {
    "OK"
}

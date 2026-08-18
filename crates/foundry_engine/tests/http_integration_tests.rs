use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use foundry_auth::JwtService;
use foundry_core::response::ApiResponse;
use foundry_engine::{AppState, build_router};
use foundry_extension::HookPipeline;
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use systems::carnival_demo::dto::ParticipateResponse;
use tower::ServiceExt;

// Create dummy uninitialized pool for offline route testing
fn dummy_app_state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://dummy:dummy@localhost:5432/dummy")
        .unwrap();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let hooks = HookPipeline::new();
    AppState::new(pool, None, jwt, hooks)
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let state = dummy_app_state();
    let subsystems = systems::register_subsystems();
    let app = build_router(state, subsystems);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_subsystem_custom_route_success() {
    let state = dummy_app_state();
    let subsystems = systems::register_subsystems();
    let app = build_router(state, subsystems);

    let payload = serde_json::json!({
        "nickname": "Tester",
        "lucky_number": 14
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/carnival_demo/participate")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: ApiResponse<ParticipateResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.code, 0);
    assert_eq!(res.data.nickname, "Tester");
    assert_eq!(res.data.lucky_number, 14);
    assert!(res.data.is_winner);
}

#[tokio::test]
async fn test_subsystem_custom_route_validation_error() {
    let state = dummy_app_state();
    let subsystems = systems::register_subsystems();
    let app = build_router(state, subsystems);

    // Invalid nickname length (< 2) and invalid lucky_number (> 100)
    let payload = serde_json::json!({
        "nickname": "T",
        "lucky_number": 999
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/carnival_demo/participate")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let err_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(err_json["code"], 42200);
    assert_eq!(err_json["i18n_key"], "errors.validation_failed");
}

#[tokio::test]
async fn test_admin_systems_auth_required() {
    let state = dummy_app_state();
    let subsystems = systems::register_subsystems();
    let app = build_router(state, subsystems);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/systems")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_platform_summary_auth_required() {
    let state = dummy_app_state();
    let subsystems = systems::register_subsystems();
    let app = build_router(state, subsystems);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/platform/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

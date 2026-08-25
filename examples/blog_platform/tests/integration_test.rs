use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use blog_platform::hooks::BlogMutationHook;
use blog_platform::systems::blog::dto::PostResponse;
use blog_platform::systems::newsletter::dto::SubscriberResponse;
use blog_platform::systems::{BlogSubsystem, NewsletterSubsystem};
use foundry::prelude::*;
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn test_app_state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://dummy:dummy@localhost:5432/dummy")
        .unwrap();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let mut hooks = HookPipeline::new();
    hooks.register(BlogMutationHook);

    let subsystems: Vec<Box<dyn SubsystemModule>> =
        vec![Box::new(BlogSubsystem), Box::new(NewsletterSubsystem)];
    AppState::new(pool, None, jwt, hooks, subsystems)
}

#[tokio::test]
async fn test_blog_platform_health_check() {
    let state = test_app_state();
    let app = foundry::engine::build_router(state);

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
async fn test_blog_subsystem_create_post_success() {
    let state = test_app_state();
    let app = foundry::engine::build_router(state);

    let payload = serde_json::json!({
        "title": "Announcing Foundry Framework 2026",
        "content": "A high-performance modular framework for modern backend architectures.",
        "author": "dev@foundkit.org",
        "tags": ["rust", "announcement"]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/blog/ext/posts")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: ApiResponse<PostResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.code, 0);
    assert_eq!(res.data.title, "Announcing Foundry Framework 2026");
    assert_eq!(res.data.author, "dev@foundkit.org");
    assert_eq!(res.data.tags.len(), 2);
}

#[tokio::test]
async fn test_blog_subsystem_validation_failure() {
    let state = test_app_state();
    let app = foundry::engine::build_router(state);

    // Title too short (< 3)
    let payload = serde_json::json!({
        "title": "Hi",
        "content": "Short",
        "author": "a"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/blog/ext/posts")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_newsletter_subsystem_subscribe_success() {
    let state = test_app_state();
    let app = foundry::engine::build_router(state);

    let payload = serde_json::json!({
        "email": "reader@example.com",
        "source": "homepage_banner"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/newsletter/ext/subscribe")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: ApiResponse<SubscriberResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.code, 0);
    assert_eq!(res.data.email, "reader@example.com");
    assert!(res.data.active);
}

#[tokio::test]
async fn test_newsletter_subsystem_invalid_email() {
    let state = test_app_state();
    let app = foundry::engine::build_router(state);

    let payload = serde_json::json!({
        "email": "not-an-email"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/newsletter/ext/subscribe")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

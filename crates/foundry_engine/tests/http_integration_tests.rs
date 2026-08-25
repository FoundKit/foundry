use axum::{
    Json, Router,
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    routing::post,
};
use foundry_auth::JwtService;
use foundry_core::{
    CustomAdminPageSpec, SubsystemModule, context::SystemContext, error::AppResult,
    response::ApiResponse,
};
use foundry_engine::{AppState, build_router};
use foundry_extension::HookPipeline;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ParticipateRequest {
    #[validate(length(min = 2, max = 32))]
    pub nickname: String,
    #[validate(range(min = 1, max = 100))]
    pub lucky_number: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipateResponse {
    pub nickname: String,
    pub lucky_number: u32,
    pub is_winner: bool,
    pub prize_name: Option<String>,
}

async fn handle_participate(
    Extension(_ctx): Extension<SystemContext>,
    Json(payload): Json<ParticipateRequest>,
) -> AppResult<Json<ApiResponse<ParticipateResponse>>> {
    payload.validate()?;
    let is_winner = payload.lucky_number % 7 == 0;
    let prize = if is_winner {
        Some("Lucky 7 Grand Prize".to_string())
    } else {
        None
    };

    Ok(Json(ApiResponse::success(ParticipateResponse {
        nickname: payload.nickname,
        lucky_number: payload.lucky_number,
        is_winner,
        prize_name: prize,
    })))
}

pub struct TestCarnivalSubsystemModule;

impl SubsystemModule for TestCarnivalSubsystemModule {
    fn slug(&self) -> &'static str {
        "carnival_demo"
    }

    fn display_name(&self) -> &'static str {
        "Carnival 2026 Demo Subsystem"
    }

    fn description(&self) -> &'static str {
        "Test carnival subsystem for integration testing"
    }

    fn register_routes(&self, router: Router) -> Router {
        router.route("/participate", post(handle_participate))
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        vec![CustomAdminPageSpec {
            key: "lottery_dashboard".to_string(),
            title: "抽奖运营大屏".to_string(),
            icon: "Gift".to_string(),
            page_type: "iframe".to_string(),
            entry: "/api/v1/s/carnival_demo/ext/custom-pages/lottery_dashboard.html".to_string(),
            required_role: None,
        }]
    }
}

// Create dummy uninitialized pool for offline route testing
fn dummy_app_state() -> AppState {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://dummy:dummy@localhost:5432/dummy")
        .unwrap();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let hooks = HookPipeline::new();
    let subsystems: Vec<Box<dyn SubsystemModule>> = vec![Box::new(TestCarnivalSubsystemModule)];
    AppState::new(pool, None, jwt, hooks, subsystems)
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let state = dummy_app_state();
    let app = build_router(state);

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
    let app = build_router(state);

    let payload = serde_json::json!({
        "nickname": "Tester",
        "lucky_number": 14
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/carnival_demo/ext/participate")
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
    let app = build_router(state);

    // Invalid nickname length (< 2) and invalid lucky_number (> 100)
    let payload = serde_json::json!({
        "nickname": "T",
        "lucky_number": 999
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/s/carnival_demo/ext/participate")
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
    let app = build_router(state);

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
    let app = build_router(state);

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

#[tokio::test]
async fn test_general_admin_forbidden_on_admins_management() {
    let state = dummy_app_state();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let token = jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "normal_admin_user",
            "admin",
            vec!["*".to_string()],
        )
        .unwrap();

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/admins")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_topic_admin_forbidden_on_platform_summary() {
    let state = dummy_app_state();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let token = jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "carnival_admin_user",
            "topic_admin",
            vec!["carnival_demo".to_string()],
        )
        .unwrap();

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/platform/summary")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_topic_admin_forbidden_on_creating_subsystems() {
    let state = dummy_app_state();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let token = jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "carnival_admin_user",
            "topic_admin",
            vec!["carnival_demo".to_string()],
        )
        .unwrap();

    let app = build_router(state);

    let payload = serde_json::json!({
        "slug": "new_subsystem",
        "name": "New Subsystem"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/systems")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_topic_admin_forbidden_on_unauthorized_topic_access() {
    let state = dummy_app_state();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let token = jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "carnival_admin_user",
            "topic_admin",
            vec!["carnival_demo".to_string()],
        )
        .unwrap();

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/s/vip_mall/details")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use foundry_auth::JwtService;
use foundry_core::SubsystemModule;
use foundry_engine::services::{
    PlatformLoginSettings, VerificationService, save_platform_login_settings,
};
use foundry_engine::{AppState, build_router};
use foundry_extension::HookPipeline;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn dummy_app_state() -> AppState {
    let db = foundry_storage::Database::dummy();
    let jwt = JwtService::new("test_secret_key_1234567890", 24);
    let hooks = HookPipeline::new();
    let subsystems: Vec<Box<dyn SubsystemModule>> = vec![];
    AppState::new(db, None, jwt, hooks, subsystems)
}

#[tokio::test]
async fn test_public_auth_settings_endpoint_default() {
    let state = dummy_app_state();
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], 0);
    assert_eq!(json["data"]["enable_password_login"], true);
    assert_eq!(json["data"]["enable_email_login"], false);
}

#[tokio::test]
async fn test_email_login_disabled_by_default() {
    let state = dummy_app_state();
    let app = build_router(state);

    let payload = serde_json::json!({
        "email": "test@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/auth/email/send-code")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("邮箱验证码登录未开启")
    );
}

#[tokio::test]
async fn test_verification_service_workflow() {
    let service = VerificationService::new(None);
    let code = "123456";

    // 1. Save code
    let save_res = service
        .save_code("bind", "admin1:test@example.com", code)
        .await;
    assert!(save_res.is_ok());

    // 2. Cooldown check - should fail within 60s
    let cooldown_res = service
        .save_code("bind", "admin1:test@example.com", "654321")
        .await;
    assert!(cooldown_res.is_err());
    assert!(cooldown_res.unwrap_err().to_string().contains("频繁"));

    // 3. Verify with wrong code - should fail
    let wrong_res = service
        .verify_code("bind", "admin1:test@example.com", "999999")
        .await;
    assert!(wrong_res.is_err());

    // 4. Verify with correct code - should succeed
    let correct_res = service
        .verify_code("bind", "admin1:test@example.com", "123456")
        .await;
    assert!(correct_res.is_ok());

    // 5. Code is one-time use - verifying again should fail
    let replay_res = service
        .verify_code("bind", "admin1:test@example.com", "123456")
        .await;
    assert!(replay_res.is_err());
}

#[test]
fn test_platform_login_settings_default() {
    let default = PlatformLoginSettings::default();
    assert!(default.enable_password_login);
    assert!(!default.enable_email_login);
    assert!(default.login_email_template.contains("{code}"));
}

#[tokio::test]
async fn test_platform_settings_validation_rules() {
    let db = foundry_storage::Database::dummy();

    // Rule 1: Cannot disable both password and email login
    let invalid_both_false = PlatformLoginSettings {
        enable_password_login: false,
        enable_email_login: false,
        login_email_subject: "主题".to_string(),
        login_email_template: "验证码: {code}".to_string(),
    };
    let res = save_platform_login_settings(&db, &invalid_both_false).await;
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("至少需要保留一种登录方式")
    );

    // Rule 2: Template must contain {code}
    let invalid_template = PlatformLoginSettings {
        enable_password_login: true,
        enable_email_login: true,
        login_email_subject: "主题".to_string(),
        login_email_template: "验证码内容没有占位符".to_string(),
    };
    let res = save_platform_login_settings(&db, &invalid_template).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("{code}"));
}

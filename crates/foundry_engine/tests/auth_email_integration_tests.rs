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
        smtp_configured: false,
        super_admin_has_logged_in: false,
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
        smtp_configured: false,
        super_admin_has_logged_in: false,
    };
    let res = save_platform_login_settings(&db, &invalid_template).await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("{code}"));

    // Rule 3: Cannot enable email login when SMTP is not configured
    foundry_engine::services::set_smtp_configured_override_for_test(Some(false));
    let invalid_smtp = PlatformLoginSettings {
        enable_password_login: true,
        enable_email_login: true,
        login_email_subject: "主题".to_string(),
        login_email_template: "验证码: {code}".to_string(),
        smtp_configured: false,
        super_admin_has_logged_in: false,
    };
    let res = save_platform_login_settings(&db, &invalid_smtp).await;
    foundry_engine::services::set_smtp_configured_override_for_test(None);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("SMTP 邮件服务未配置"));
}

#[tokio::test]
async fn test_send_email_intercepted_when_smtp_not_configured() {
    foundry_engine::services::set_smtp_configured_override_for_test(Some(false));
    let res =
        foundry_engine::services::send_email("user@example.com", "Test Subject", "Test Content")
            .await;
    foundry_engine::services::set_smtp_configured_override_for_test(None);

    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("SMTP 邮件服务未配置"));
}

#[tokio::test]
async fn test_bind_email_send_code_intercepted_when_smtp_not_configured() {
    foundry_engine::services::set_smtp_configured_override_for_test(Some(false));
    let state = dummy_app_state();
    let app = build_router(state);

    let token = dummy_app_state()
        .jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "admin",
            "super_admin",
            vec!["*".to_string()],
        )
        .unwrap();

    let payload = serde_json::json!({
        "email": "admin_bind@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/auth/bind-email/send-code")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    foundry_engine::services::set_smtp_configured_override_for_test(None);

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("SMTP 邮件服务未配置")
    );
}

#[tokio::test]
async fn test_smtp_configuration_lifecycle() {
    let db = foundry_storage::Database::dummy();

    foundry_engine::services::set_smtp_configured_override_for_test(Some(true));
    assert!(foundry_engine::services::is_smtp_configured());

    let valid_settings = PlatformLoginSettings {
        enable_password_login: true,
        enable_email_login: true,
        login_email_subject: "【测试】登录验证码".to_string(),
        login_email_template: "验证码: {code}".to_string(),
        smtp_configured: true,
        super_admin_has_logged_in: false,
    };
    let res = save_platform_login_settings(&db, &valid_settings).await;
    // Validation passes when SMTP is configured (any error here is from dummy DB pool, not validation)
    if let Err(foundry_core::error::AppError::Validation(ref msg)) = res {
        panic!("Validation unexpectedly failed with: {}", msg);
    }

    foundry_engine::services::set_smtp_configured_override_for_test(Some(false));
    assert!(!foundry_engine::services::is_smtp_configured());
    let res_no_smtp = save_platform_login_settings(&db, &valid_settings).await;
    assert!(matches!(
        res_no_smtp,
        Err(foundry_core::error::AppError::Validation(ref msg)) if msg.contains("SMTP 邮件服务未配置")
    ));
    foundry_engine::services::set_smtp_configured_override_for_test(None);
}

#[tokio::test]
async fn test_change_password_endpoint_auth_required() {
    let state = dummy_app_state();
    let app = build_router(state);

    let payload = serde_json::json!({
        "old_password": "admin123456",
        "new_password": "new_secret_password"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/auth/password")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_change_password_endpoint_validation_rules() {
    let state = dummy_app_state();
    let app = build_router(state.clone());

    let token = state
        .jwt
        .generate_token(
            uuid::Uuid::new_v4(),
            "admin",
            "super_admin",
            vec!["*".to_string()],
        )
        .unwrap();

    // 1. Password too short (< 6 chars)
    let payload_short = serde_json::json!({
        "old_password": "admin123456",
        "new_password": "123"
    });

    let res_short = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/auth/password")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_vec(&payload_short).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_short.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 2. Same password as old
    let payload_same = serde_json::json!({
        "old_password": "admin123456",
        "new_password": "admin123456"
    });

    let res_same = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/auth/password")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_vec(&payload_same).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_same.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

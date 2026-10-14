use crate::services::{
    PlatformLoginSettings, VerificationService, get_platform_login_settings, is_smtp_configured,
    save_platform_login_settings, send_email,
};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Extension, State},
};
use foundry_auth::{AdminClaims, verify_password};
use foundry_core::error::{AppError, AppResult};
use foundry_core::response::ApiResponse;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct PublicAuthSettingsResponse {
    pub enable_password_login: bool,
    pub enable_email_login: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendLoginCodeRequest {
    #[validate(email(message = "Invalid email address format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmailLoginRequest {
    #[validate(email(message = "Invalid email address format"))]
    pub email: String,
    #[validate(length(min = 4, max = 10, message = "Verification code length invalid"))]
    pub code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendBindEmailCodeRequest {
    #[validate(email(message = "Invalid email address format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyBindEmailRequest {
    #[validate(email(message = "Invalid email address format"))]
    pub email: String,
    #[validate(length(min = 4, max = 10, message = "Verification code length invalid"))]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: AdminProfile,
}

#[derive(Debug, Serialize)]
pub struct AdminProfile {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub allowed_systems: serde_json::Value,
}

/// GET /api/v1/admin/auth/settings (Public auth configuration: which login methods are enabled)
pub async fn public_auth_settings_handler(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<PublicAuthSettingsResponse>>> {
    let settings = get_platform_login_settings(&state.db).await;
    let enable_email_login = settings.enable_email_login && is_smtp_configured();
    Ok(Json(ApiResponse::success(PublicAuthSettingsResponse {
        enable_password_login: settings.enable_password_login,
        enable_email_login,
    })))
}

/// POST /api/v1/admin/auth/login (Account and password login)
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    payload.validate()?;

    let settings = get_platform_login_settings(&state.db).await;
    if !settings.enable_password_login {
        return Err(AppError::Forbidden(
            "账号密码登录已关闭，请使用邮箱验证码登录 (Account password login is disabled, please use email verification code login)"
                .to_string(),
        ));
    }

    let admin = state
        .db
        .admins()
        .get_by_username(&payload.username)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if admin.status != 1 {
        return Err(AppError::Forbidden("Account has been disabled".to_string()));
    }

    let is_valid = verify_password(&payload.password, &admin.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let allowed: Vec<String> =
        serde_json::from_value(admin.allowed_systems.clone()).unwrap_or_default();

    let token = state
        .jwt
        .generate_token(admin.id, &admin.username, &admin.role, allowed)?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        admin: profile,
    })))
}

/// POST /api/v1/admin/auth/email/send-code (Public: send verification code for login)
pub async fn send_login_code_handler(
    State(state): State<AppState>,
    Json(payload): Json<SendLoginCodeRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate()?;

    let settings = get_platform_login_settings(&state.db).await;
    if !settings.enable_email_login {
        return Err(AppError::Forbidden(
            "邮箱验证码登录未开启 (Email verification code login is disabled)".to_string(),
        ));
    }

    if !is_smtp_configured() {
        return Err(AppError::BadRequest(
            "SMTP 邮件服务未配置，无法发送登录验证码 (SMTP email service is not configured)"
                .to_string(),
        ));
    }

    let email = payload.email.trim();
    let admin = state.db.admins().get_by_email(email).await.map_err(|_| {
        AppError::NotFound(
            "该邮箱未绑定任何管理员账号 (No administrator account is associated with this email)"
                .to_string(),
        )
    })?;

    if admin.status != 1 {
        return Err(AppError::Forbidden(
            "该管理员账号已被禁用 (Account has been disabled)".to_string(),
        ));
    }

    let code = VerificationService::generate_code();
    state.verification.save_code("login", email, &code).await?;

    let subject = &settings.login_email_subject;
    let body = settings.login_email_template.replace("{code}", &code);

    send_email(email, subject, &body).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "验证码已发送至您的邮箱，5分钟内有效 (Verification code sent to your email, valid for 5 minutes)"
    }))))
}

/// POST /api/v1/admin/auth/email/login (Public: login using email verification code)
pub async fn email_login_handler(
    State(state): State<AppState>,
    Json(payload): Json<EmailLoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    payload.validate()?;

    let settings = get_platform_login_settings(&state.db).await;
    if !settings.enable_email_login {
        return Err(AppError::Forbidden(
            "邮箱验证码登录未开启 (Email verification code login is disabled)".to_string(),
        ));
    }

    if !is_smtp_configured() {
        return Err(AppError::BadRequest(
            "SMTP 邮件服务未配置，无法使用邮箱验证码登录 (SMTP email service is not configured)"
                .to_string(),
        ));
    }

    let email = payload.email.trim();
    state
        .verification
        .verify_code("login", email, &payload.code)
        .await?;

    let admin = state.db.admins().get_by_email(email).await.map_err(|_| {
        AppError::NotFound(
            "该邮箱未绑定任何管理员账号 (No administrator account is associated with this email)"
                .to_string(),
        )
    })?;

    if admin.status != 1 {
        return Err(AppError::Forbidden("Account has been disabled".to_string()));
    }

    let allowed: Vec<String> =
        serde_json::from_value(admin.allowed_systems.clone()).unwrap_or_default();

    let token = state
        .jwt
        .generate_token(admin.id, &admin.username, &admin.role, allowed)?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        admin: profile,
    })))
}

/// POST /api/v1/admin/auth/bind-email/send-code (Protected: send code to bind email)
pub async fn send_bind_email_code_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<SendBindEmailCodeRequest>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    payload.validate()?;
    let email = payload.email.trim();

    if !is_smtp_configured() {
        return Err(AppError::BadRequest(
            "SMTP 邮件服务未配置，无法发送邮箱绑定验证码 (SMTP email service is not configured)"
                .to_string(),
        ));
    }

    // Check if email is already bound to another admin
    if matches!(state.db.admins().get_by_email(email).await, Ok(existing) if existing.id != claims.sub)
    {
        return Err(AppError::Conflict(
            "该邮箱已被其他管理员绑定，请使用其他邮箱 (This email is already bound to another administrator)"
                .to_string(),
        ));
    }

    let code = VerificationService::generate_code();
    let identifier = format!("{}:{}", claims.sub, email);
    state
        .verification
        .save_code("bind", &identifier, &code)
        .await?;

    let subject = "【Foundry】管理员邮箱绑定验证码";
    let body = format!(
        "您好！您正在绑定 Foundry 管理员账号（{}）的邮箱，您的验证码是：{}，5分钟内有效。如非本人操作请忽略。\n\nHello! You are binding email for administrator account ({}), verification code: {}, valid for 5 minutes.",
        claims.username, code, claims.username, code
    );

    send_email(email, subject, &body).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "message": "绑定验证码已发送至您的邮箱，5分钟内有效 (Verification code sent to your email)"
    }))))
}

/// POST /api/v1/admin/auth/bind-email/verify (Protected: verify code and bind email)
pub async fn verify_bind_email_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<VerifyBindEmailRequest>,
) -> AppResult<Json<ApiResponse<AdminProfile>>> {
    payload.validate()?;
    let email = payload.email.trim();

    if !is_smtp_configured() {
        return Err(AppError::BadRequest(
            "SMTP 邮件服务未配置，无法验证邮箱绑定 (SMTP email service is not configured)"
                .to_string(),
        ));
    }

    let identifier = format!("{}:{}", claims.sub, email);
    state
        .verification
        .verify_code("bind", &identifier, &payload.code)
        .await?;

    // Check conflict again to prevent race conditions
    if matches!(state.db.admins().get_by_email(email).await, Ok(existing) if existing.id != claims.sub)
    {
        return Err(AppError::Conflict(
            "该邮箱已被其他管理员绑定，请使用其他邮箱 (This email is already bound to another administrator)"
                .to_string(),
        ));
    }

    let admin = state
        .db
        .admins()
        .update(claims.sub, Some(email), None, None, None, None)
        .await?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(profile)))
}

/// GET /api/v1/admin/platform/login-settings (Protected: Super Admin view platform login settings)
pub async fn get_login_settings_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<PlatformLoginSettings>>> {
    if !claims.is_super_admin() {
        return Err(AppError::Forbidden(
            "只有超级管理员可以查看平台登录配置 (Only Super Admin can view login settings)"
                .to_string(),
        ));
    }

    let settings = get_platform_login_settings(&state.db).await;
    Ok(Json(ApiResponse::success(settings)))
}

/// PUT /api/v1/admin/platform/login-settings (Protected: Super Admin update platform login settings)
pub async fn update_login_settings_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
    Json(payload): Json<PlatformLoginSettings>,
) -> AppResult<Json<ApiResponse<PlatformLoginSettings>>> {
    if !claims.is_super_admin() {
        return Err(AppError::Forbidden(
            "只有超级管理员可以修改平台登录配置 (Only Super Admin can update login settings)"
                .to_string(),
        ));
    }

    save_platform_login_settings(&state.db, &payload).await?;
    let mut resp = payload;
    resp.smtp_configured = is_smtp_configured();
    Ok(Json(ApiResponse::success(resp)))
}

/// GET /api/v1/admin/auth/me (Protected: get current admin profile)
pub async fn me_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AdminClaims>,
) -> AppResult<Json<ApiResponse<AdminProfile>>> {
    let admin = state.db.admins().get_by_id(claims.sub).await?;

    let profile = AdminProfile {
        id: admin.id,
        username: admin.username,
        email: admin.email,
        role: admin.role,
        allowed_systems: admin.allowed_systems,
    };

    Ok(Json(ApiResponse::success(profile)))
}

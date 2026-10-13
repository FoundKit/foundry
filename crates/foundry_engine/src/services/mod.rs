pub mod email;
pub mod verification;

pub use email::send_email;
pub use verification::VerificationService;

use foundry_core::error::{AppError, AppResult};
use foundry_storage::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformLoginSettings {
    pub enable_password_login: bool,
    pub enable_email_login: bool,
    pub login_email_subject: String,
    pub login_email_template: String,
}

impl Default for PlatformLoginSettings {
    fn default() -> Self {
        Self {
            enable_password_login: true,
            enable_email_login: false,
            login_email_subject: "【Foundry】登录验证码".to_string(),
            login_email_template: "您的登录验证码是：{code}，5分钟内有效。请勿将验证码泄露给他人。如非本人操作请忽略。".to_string(),
        }
    }
}

pub async fn get_platform_login_settings(db: &Database) -> PlatformLoginSettings {
    let mut settings = PlatformLoginSettings::default();
    if let Ok(configs) = db.configs().list("_platform").await {
        for cfg in configs {
            match cfg.key.as_str() {
                "enable_password_login" => {
                    if let Some(v) = cfg.value.and_then(|v| v.as_bool()) {
                        settings.enable_password_login = v;
                    }
                }
                "enable_email_login" => {
                    if let Some(v) = cfg.value.and_then(|v| v.as_bool()) {
                        settings.enable_email_login = v;
                    }
                }
                "login_email_subject" => {
                    if let Some(v) = cfg
                        .value
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .filter(|s| !s.trim().is_empty())
                    {
                        settings.login_email_subject = v;
                    }
                }
                "login_email_template" => {
                    if let Some(v) = cfg
                        .value
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .filter(|s| !s.trim().is_empty())
                    {
                        settings.login_email_template = v;
                    }
                }
                _ => {}
            }
        }
    }
    settings
}

pub async fn save_platform_login_settings(
    db: &Database,
    settings: &PlatformLoginSettings,
) -> AppResult<()> {
    if !settings.enable_password_login && !settings.enable_email_login {
        return Err(AppError::Validation(
            "至少需要保留一种登录方式（账号密码登录或邮箱验证码登录） (At least one login method must be enabled)"
                .to_string(),
        ));
    }

    if !settings.login_email_template.contains("{code}") {
        return Err(AppError::Validation(
            "登录邮件内容必须包含验证码占位符 {code} (Login email template must contain {code} placeholder)"
                .to_string(),
        ));
    }

    if settings.login_email_subject.trim().is_empty() {
        return Err(AppError::Validation(
            "登录邮件主题不能为空 (Login email subject cannot be empty)".to_string(),
        ));
    }

    db.configs()
        .upsert(
            "_platform",
            "enable_password_login",
            "账号密码登录",
            "boolean",
            Some(serde_json::Value::Bool(settings.enable_password_login)),
            serde_json::json!({}),
            1,
        )
        .await?;

    db.configs()
        .upsert(
            "_platform",
            "enable_email_login",
            "邮箱验证码登录",
            "boolean",
            Some(serde_json::Value::Bool(settings.enable_email_login)),
            serde_json::json!({}),
            2,
        )
        .await?;

    db.configs()
        .upsert(
            "_platform",
            "login_email_subject",
            "登录邮件主题",
            "string",
            Some(serde_json::Value::String(
                settings.login_email_subject.clone(),
            )),
            serde_json::json!({}),
            3,
        )
        .await?;

    db.configs()
        .upsert(
            "_platform",
            "login_email_template",
            "登录邮件模板",
            "richtext",
            Some(serde_json::Value::String(
                settings.login_email_template.clone(),
            )),
            serde_json::json!({}),
            4,
        )
        .await?;

    Ok(())
}

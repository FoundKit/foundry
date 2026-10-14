use foundry_core::error::{AppError, AppResult};
use std::sync::atomic::{AtomicI8, Ordering};

static SMTP_CONFIGURED_OVERRIDE: AtomicI8 = AtomicI8::new(0);

/// Check whether SMTP email service is configured via environment variables.
pub fn is_smtp_configured() -> bool {
    match SMTP_CONFIGURED_OVERRIDE.load(Ordering::Relaxed) {
        1 => true,
        -1 => false,
        _ => std::env::var("SMTP_HOST")
            .ok()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false),
    }
}

/// Helper for tests to simulate SMTP configured state without modifying environment variables.
#[doc(hidden)]
pub fn set_smtp_configured_override_for_test(configured: Option<bool>) {
    let val = match configured {
        Some(true) => 1,
        Some(false) => -1,
        None => 0,
    };
    SMTP_CONFIGURED_OVERRIDE.store(val, Ordering::Relaxed);
}

pub async fn send_email(to: &str, subject: &str, body: &str) -> AppResult<()> {
    let smtp_host = std::env::var("SMTP_HOST")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let Some(host) = smtp_host else {
        tracing::warn!(
            "Attempted to send email to {}, but SMTP is not configured. Email operation intercepted.",
            to
        );
        return Err(AppError::BadRequest(
            "SMTP 邮件服务未配置，无法发送邮件 (SMTP email service is not configured)".to_string(),
        ));
    };

    let port: u16 = std::env::var("SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let user = std::env::var("SMTP_USER").unwrap_or_default();
    let pass = std::env::var("SMTP_PASS").unwrap_or_default();
    let from = std::env::var("SMTP_FROM")
        .unwrap_or_else(|_| "Foundry <no-reply@foundry.local>".to_string());

    let email_msg = lettre::Message::builder()
        .from(
            from.parse()
                .map_err(|e| AppError::Internal(format!("Invalid SMTP from address: {}", e)))?,
        )
        .to(to
            .parse()
            .map_err(|e| AppError::Validation(format!("Invalid email address: {}", e)))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to build email message: {}", e)))?;

    let mut transport_builder = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&host)
        .map_err(|e| AppError::Internal(format!("Failed to create SMTP transport: {}", e)))?
        .port(port);

    if !user.is_empty() && !pass.is_empty() {
        let creds = lettre::transport::smtp::authentication::Credentials::new(user, pass);
        transport_builder = transport_builder.credentials(creds);
    }

    let transport = transport_builder.build();
    use lettre::AsyncTransport;
    transport
        .send(email_msg)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to send email via SMTP: {}", e)))?;

    tracing::info!(
        "Successfully sent email to {} via SMTP relay {}:{}",
        to,
        host,
        port
    );

    Ok(())
}

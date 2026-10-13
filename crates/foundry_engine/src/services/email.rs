use foundry_core::error::{AppError, AppResult};

pub async fn send_email(to: &str, subject: &str, body: &str) -> AppResult<()> {
    let smtp_host = std::env::var("SMTP_HOST")
        .ok()
        .filter(|s| !s.trim().is_empty());

    if let Some(host) = smtp_host {
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

        let mut transport_builder =
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&host)
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
    } else {
        // Dev / Test mock mode: Log clearly so developers/testers can see the code immediately
        tracing::info!(
            "📧 [EMAIL DEV MODE] SMTP_HOST not configured. Email logged to console:\nTo: {}\nSubject: {}\nContent:\n{}",
            to,
            subject,
            body
        );
    }

    Ok(())
}

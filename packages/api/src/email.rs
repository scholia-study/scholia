use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::config::AppConfig;

pub async fn send_verification_email(
    config: &AppConfig,
    to: &str,
    token: &str,
) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let link = format!("{}/auth/verify-email?token={}", config.backend_url, token);

    let html = format!(
        r#"<h2>Verify your email</h2>
<p>Click the link below to verify your email address:</p>
<p><a href="{link}">Verify Email</a></p>
<p>This link expires in 24 hours.</p>
<p>If you didn't create an account, you can ignore this email.</p>"#
    );

    let email =
        CreateEmailBaseOptions::new(&config.from_email, [to], "Verify your email").with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| format!("Failed to send verification email: {e}"))?;

    Ok(())
}

pub async fn send_password_reset_email(
    config: &AppConfig,
    to: &str,
    token: &str,
) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let link = format!("{}/reset-password?token={}", config.frontend_url, token);

    let html = format!(
        r#"<h2>Reset your password</h2>
<p>Click the link below to reset your password:</p>
<p><a href="{link}">Reset Password</a></p>
<p>This link expires in 1 hour.</p>
<p>If you didn't request this, you can ignore this email.</p>"#
    );

    let email = CreateEmailBaseOptions::new(&config.from_email, [to], "Reset your password")
        .with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| format!("Failed to send password reset email: {e}"))?;

    Ok(())
}

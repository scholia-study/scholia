use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::system::config::AppConfig;

pub async fn send_verification_email(
    config: &AppConfig,
    to: &str,
    token: &str,
) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let link = format!(
        "{}/api/auth/verify-email?token={}",
        config.backend_url, token
    );

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

/// Sent when someone tries to register with an email that already has an
/// account — the registration response itself stays indistinguishable
/// from a fresh signup, so this email is the only channel that reveals
/// (to the address owner alone) what happened.
pub async fn send_account_exists_email(config: &AppConfig, to: &str) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let link = format!("{}/forgot-password", config.frontend_url);

    let html = format!(
        r#"<h2>You already have an account</h2>
<p>Someone (hopefully you) tried to create a Scholia account with this email address, but an account already exists.</p>
<p>If this was you, just log in — and if you've forgotten your password, you can <a href="{link}">reset it here</a>.</p>
<p>If this wasn't you, no action is needed; your account is unaffected.</p>"#
    );

    let email =
        CreateEmailBaseOptions::new(&config.from_email, [to], "You already have an account")
            .with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| format!("Failed to send account-exists email: {e}"))?;

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

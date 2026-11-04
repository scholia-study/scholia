use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::system::config::AppConfig;

const ADMIN_NOTIFICATION_EMAIL: &str = "contact@filipniklas.com";

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

pub struct NewFeedbackNotice {
    pub feedback_id: String,
    pub submitter_name: String,
    pub submitter_email: String,
    pub body: String,
    pub url: Option<String>,
}

pub struct NewReviewRequestNotice {
    pub article_title: String,
    pub author_name: String,
    pub author_email: String,
    pub intent: &'static str,
    pub message: Option<String>,
}

/// Unlike the account emails above, these templates carry user-written text,
/// so every submitted field goes through `clean_text` — including the page URL,
/// which is rendered as text so a submission can't plant a link in the inbox.
fn new_feedback_html(frontend_url: &str, notice: &NewFeedbackNotice) -> String {
    let submitter = ammonia::clean_text(&notice.submitter_name);
    let email = ammonia::clean_text(&notice.submitter_email);
    let body = ammonia::clean_text(&notice.body);
    let page = notice
        .url
        .as_deref()
        .map(|url| format!("<p>Page: {}</p>", ammonia::clean_text(url)))
        .unwrap_or_default();
    let link = format!("{frontend_url}/manage/feedback/{}", notice.feedback_id);

    format!(
        r#"<h2>New feedback</h2>
<p>From {submitter} ({email})</p>
{page}
<p style="white-space: pre-wrap">{body}</p>
<p><a href="{link}">Open in Scholia</a></p>"#
    )
}

fn new_review_request_html(frontend_url: &str, notice: &NewReviewRequestNotice) -> String {
    let title = ammonia::clean_text(&notice.article_title);
    let author = ammonia::clean_text(&notice.author_name);
    let email = ammonia::clean_text(&notice.author_email);
    let intent = notice.intent;
    let message = notice
        .message
        .as_deref()
        .map(|message| {
            format!(
                r#"<p style="white-space: pre-wrap">{}</p>"#,
                ammonia::clean_text(message)
            )
        })
        .unwrap_or_default();
    let link = format!("{frontend_url}/manage/article-reviews");

    format!(
        r#"<h2>New article review request</h2>
<p><strong>{title}</strong></p>
<p>From {author} ({email}), requesting {intent} review</p>
{message}
<p><a href="{link}">Open the review queue</a></p>"#
    )
}

pub async fn send_new_feedback_email(
    config: &AppConfig,
    notice: &NewFeedbackNotice,
) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let html = new_feedback_html(&config.frontend_url, notice);

    let email = CreateEmailBaseOptions::new(
        &config.from_email,
        [ADMIN_NOTIFICATION_EMAIL],
        "New feedback",
    )
    .with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| format!("Failed to send new-feedback email: {e}"))?;

    Ok(())
}

pub async fn send_new_review_request_email(
    config: &AppConfig,
    notice: &NewReviewRequestNotice,
) -> Result<(), String> {
    let resend = Resend::new(&config.resend_api_key);
    let html = new_review_request_html(&config.frontend_url, notice);
    let subject = format!("Review request: {}", notice.article_title);

    let email =
        CreateEmailBaseOptions::new(&config.from_email, [ADMIN_NOTIFICATION_EMAIL], &subject)
            .with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| format!("Failed to send new-review-request email: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOSTILE: &str = "<a href=\"https://evil.example\">click</a>";

    #[test]
    fn feedback_html_escapes_every_submitted_field() {
        let html = new_feedback_html(
            "https://scholia.test",
            &NewFeedbackNotice {
                feedback_id: "abc".into(),
                submitter_name: HOSTILE.into(),
                submitter_email: HOSTILE.into(),
                body: HOSTILE.into(),
                url: Some(HOSTILE.into()),
            },
        );
        assert!(!html.contains("evil.example\">"));
        assert!(!html.contains("<a href=\"https://evil"));
        assert!(html.contains(r#"<a href="https://scholia.test/manage/feedback/abc">"#));
    }

    #[test]
    fn review_request_html_escapes_every_submitted_field() {
        let html = new_review_request_html(
            "https://scholia.test",
            &NewReviewRequestNotice {
                article_title: HOSTILE.into(),
                author_name: HOSTILE.into(),
                author_email: HOSTILE.into(),
                intent: "publication",
                message: Some(HOSTILE.into()),
            },
        );
        assert!(!html.contains("<a href=\"https://evil"));
        assert!(html.contains(r#"<a href="https://scholia.test/manage/article-reviews">"#));
    }
}

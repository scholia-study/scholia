//! Fire-and-forget operational alerts to the shared ntfy topic (the same
//! `NTFY_URL` secret the backup cronjob publishes to). Alerts are for the
//! rare events an operator should see on their phone — circuit breakers,
//! not routine errors (those go to Sentry).

use crate::system::config::AppConfig;

pub enum Priority {
    High,
    Urgent,
}

impl Priority {
    fn header_value(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

/// Post `message` to the ntfy topic. No-op when `NTFY_URL` is unset
/// (local dev). Failures are logged and swallowed — an alert must never
/// take the request path down with it.
pub fn alert(config: &AppConfig, title: &str, message: &str, priority: Priority) {
    let Some(url) = config.ntfy_url.clone() else {
        return;
    };
    let title = title.to_owned();
    let message = message.to_owned();
    tokio::spawn(async move {
        let result = reqwest::Client::new()
            .post(&url)
            .header("Title", &title)
            .header("Priority", priority.header_value())
            .body(message)
            .send()
            .await;
        match result {
            Ok(resp) if !resp.status().is_success() => {
                tracing::warn!("ntfy alert '{title}' → {}", resp.status());
            }
            Err(e) => tracing::warn!("ntfy alert '{title}' failed: {e}"),
            Ok(_) => {}
        }
    });
}

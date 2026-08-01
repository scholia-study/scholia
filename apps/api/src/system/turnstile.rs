use serde::Deserialize;

const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Deserialize)]
struct SiteverifyResponse {
    success: bool,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

pub async fn verify(secret: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let result = reqwest::Client::new()
        .post(SITEVERIFY_URL)
        .form(&[("secret", secret), ("response", token)])
        .send()
        .await;
    match result {
        Ok(resp) => match resp.json::<SiteverifyResponse>().await {
            Ok(body) => {
                if !body.success {
                    tracing::warn!("Turnstile verification rejected: {:?}", body.error_codes);
                }
                body.success
            }
            Err(e) => {
                tracing::error!("Turnstile siteverify returned unparseable body: {e}");
                false
            }
        },
        Err(e) => {
            tracing::error!("Turnstile siteverify request failed: {e}");
            false
        }
    }
}

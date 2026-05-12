use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub session_secret: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub backend_url: String,
    pub frontend_url: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub stripe_api_key: String,
    pub stripe_webhook_secret: String,
    pub stripe_price_base: String,
    pub stripe_price_mid: String,
    pub stripe_price_high: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let cfg = Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            session_secret: env::var("SESSION_SECRET").expect("SESSION_SECRET must be set"),
            resend_api_key: env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set"),
            from_email: env::var("FROM_EMAIL").expect("FROM_EMAIL must be set"),
            backend_url: env::var("BACKEND_URL").expect("BACKEND_URL must be set"),
            frontend_url: env::var("FRONTEND_URL").expect("FRONTEND_URL must be set"),
            github_client_id: env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
            github_redirect_uri: env::var("GITHUB_REDIRECT_URI")
                .expect("GITHUB_REDIRECT_URI must be set"),
            stripe_api_key: env::var("STRIPE_API_KEY").expect("STRIPE_API_KEY must be set"),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                .expect("STRIPE_WEBHOOK_SECRET must be set"),
            stripe_price_base: env::var("STRIPE_PRICE_BASE")
                .expect("STRIPE_PRICE_BASE must be set"),
            stripe_price_mid: env::var("STRIPE_PRICE_MID").expect("STRIPE_PRICE_MID must be set"),
            stripe_price_high: env::var("STRIPE_PRICE_HIGH")
                .expect("STRIPE_PRICE_HIGH must be set"),
        };
        cfg.verify_stripe_prices();
        cfg
    }

    /// Sanity-check the configured Stripe Price IDs at boot.
    /// Catches the most common mistake: pasting a Product ID
    /// (`prod_…`) or a test/live mismatch into the wrong env. Does
    /// not call the Stripe API — that's a future enhancement.
    fn verify_stripe_prices(&self) {
        for (name, value) in [
            ("STRIPE_PRICE_BASE", &self.stripe_price_base),
            ("STRIPE_PRICE_MID", &self.stripe_price_mid),
            ("STRIPE_PRICE_HIGH", &self.stripe_price_high),
        ] {
            assert!(
                value.starts_with("price_"),
                "{name} must start with `price_` (got {value:?})"
            );
        }
    }

    /// Map a Stripe Price ID back to its internal tier name.
    /// Returns the role name to assign; `None` for an unknown price
    /// (e.g. a legacy sub on a removed tier — webhook should log + ack).
    pub fn role_for_price_id(&self, price_id: &str) -> Option<&'static str> {
        if price_id == self.stripe_price_base {
            Some("scholiast")
        } else if price_id == self.stripe_price_mid {
            Some("scholiast_benefactor")
        } else if price_id == self.stripe_price_high {
            Some("scholiast_patron")
        } else {
            None
        }
    }

    /// Map a Stripe Price ID to the short tier label stored in
    /// `subscriptions.tier` ('base' | 'mid' | 'high').
    pub fn tier_label_for_price_id(&self, price_id: &str) -> Option<&'static str> {
        if price_id == self.stripe_price_base {
            Some("base")
        } else if price_id == self.stripe_price_mid {
            Some("mid")
        } else if price_id == self.stripe_price_high {
            Some("high")
        } else {
            None
        }
    }
}

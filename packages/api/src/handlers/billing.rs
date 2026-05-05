use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use stripe_billing::billing_portal_session::CreateBillingPortalSession;
use stripe_billing::subscription::RetrieveSubscription;
use stripe_checkout::checkout_session::{CreateCheckoutSession, CreateCheckoutSessionLineItems};
use stripe_core::customer::CreateCustomer;
use stripe_shared::{CheckoutSessionMode, CheckoutSessionUiMode};
use stripe_webhook::{EventObject, Webhook};
// `Subscription` lives in stripe_shared (the cross-crate model crate).
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::state::AppState;

// ── Request / response types ────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct CreateCheckoutRequest {
    /// Tier slug: "base" | "mid" | "high".
    pub tier: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateCheckoutResponse {
    /// Pass to the Stripe Embedded Checkout React component.
    pub client_secret: String,
}

#[derive(Serialize, ToSchema)]
pub struct PortalSessionResponse {
    /// Stripe-hosted Customer Portal URL; redirect the browser here.
    pub url: String,
}

// ── Handlers ────────────────────────────────────────────────

/// Create a Stripe Embedded Checkout Session for the given tier.
/// Lazily creates the user's Stripe customer on first call.
#[utoipa::path(
    post,
    path = "/api/billing/checkout",
    request_body = CreateCheckoutRequest,
    responses(
        (status = 200, description = "Checkout session created", body = CreateCheckoutResponse),
        (status = 400, description = "Invalid tier"),
        (status = 401, description = "Not authenticated"),
        (status = 500, description = "Stripe API error")
    ),
    tag = "billing"
)]
pub async fn create_checkout_session(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCheckoutRequest>,
) -> Response {
    let price_id = match req.tier.as_str() {
        "base" => state.config.stripe_price_base.clone(),
        "mid" => state.config.stripe_price_mid.clone(),
        "high" => state.config.stripe_price_high.clone(),
        _ => return AppError::BadRequest("invalid tier".into()).into_response(),
    };

    let customer_id = match get_or_create_stripe_customer(&state, &user).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    // Stripe Embedded Checkout's `redirect_on_completion` defaults to
    // `always` — after success, Stripe pushes the user to `return_url`.
    // The dedicated welcome page handles polling /api/auth/me until the
    // webhook lands the new role, then renders a thank-you state.
    let return_url = format!("{}/membership/welcome", state.config.frontend_url);

    let line_items = vec![CreateCheckoutSessionLineItems {
        price: Some(price_id),
        quantity: Some(1),
        ..Default::default()
    }];

    // Stripe renamed `embedded` -> `embedded_page` in 2025; the
    // async-stripe v1.0.0-rc.5 enum reflects the new value, which is
    // what we want.
    let result = CreateCheckoutSession::new()
        .ui_mode(CheckoutSessionUiMode::EmbeddedPage)
        .mode(CheckoutSessionMode::Subscription)
        .customer(customer_id)
        .line_items(line_items)
        .return_url(return_url)
        .send(&state.stripe)
        .await;

    let session = match result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("stripe checkout.create failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let client_secret = match session.client_secret {
        Some(s) => s,
        None => {
            tracing::error!("stripe checkout.create returned no client_secret");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(CreateCheckoutResponse { client_secret }).into_response()
}

/// Create a Stripe Customer Portal session and return the redirect URL.
/// Requires the user to already have a `stripe_customer_id` (i.e. has
/// initiated at least one checkout). 404 otherwise.
#[utoipa::path(
    post,
    path = "/api/billing/portal",
    responses(
        (status = 200, description = "Portal session created", body = PortalSessionResponse),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "User has no Stripe customer yet"),
        (status = 500, description = "Stripe API error")
    ),
    tag = "billing"
)]
pub async fn create_portal_session(State(state): State<AppState>, user: AuthUser) -> Response {
    let customer_id: Option<String> =
        match sqlx::query_scalar("SELECT stripe_customer_id FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await
        {
            Ok(v) => v,
            Err(e) => return AppError::from(e).into_response(),
        };

    let customer_id = match customer_id {
        Some(id) => id,
        None => {
            return AppError::NotFound("no Stripe customer for this user".into()).into_response();
        }
    };

    let return_url = format!("{}/membership", state.config.frontend_url);

    let result = CreateBillingPortalSession::new()
        .customer(customer_id)
        .return_url(return_url)
        .send(&state.stripe)
        .await;

    let session = match result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("stripe billing_portal.create failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    Json(PortalSessionResponse { url: session.url }).into_response()
}

/// Stripe webhook endpoint. Public route — signature verification is
/// the only auth. Subscribes to:
///   - customer.subscription.created
///   - customer.subscription.updated
///   - customer.subscription.deleted
///
/// Idempotency strategy: dedup by event ID, then refetch the
/// subscription from the Stripe API (defeats out-of-order delivery),
/// then upsert the row + sync user_roles in one transaction.
pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let sig = match headers
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
    {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "missing signature").into_response(),
    };
    let payload = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "non-utf8 body").into_response(),
    };

    let event = match Webhook::construct_event(payload, sig, &state.config.stripe_webhook_secret) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("stripe webhook signature verification failed: {e}");
            return (StatusCode::BAD_REQUEST, "bad signature").into_response();
        }
    };

    // Extract subscription ID from any of the three events we care about.
    let sub_id: Option<String> = match &event.data.object {
        EventObject::CustomerSubscriptionCreated(s)
        | EventObject::CustomerSubscriptionUpdated(s)
        | EventObject::CustomerSubscriptionDeleted(s) => Some(s.id.to_string()),
        _ => {
            // Other event types — ack and ignore.
            return StatusCode::OK.into_response();
        }
    };

    let Some(sub_id) = sub_id else {
        return StatusCode::OK.into_response();
    };

    // Refetch the subscription from Stripe's API to defeat out-of-order
    // delivery. The webhook payload could be stale by the time we
    // process it; the API always returns the current state.
    let sub = match RetrieveSubscription::new(sub_id.as_str())
        .send(&state.stripe)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("stripe subscription.retrieve({sub_id}) failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Apply the (possibly fresher) state to our DB. Idempotent —
    // duplicate webhook deliveries land on the same target state.
    if let Err(e) =
        apply_subscription(&state.pool, &state.config, &event.id.to_string(), &sub).await
    {
        tracing::error!("apply_subscription({sub_id}) failed: {e:?}");
        // Return 5xx so Stripe retries. INSERT into stripe_processed_events
        // is part of the same transaction, so a retried event will rerun.
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}

// ── Helpers ─────────────────────────────────────────────────

/// Lazily resolve the user's Stripe customer ID, creating one at
/// Stripe if this is their first checkout. Stores the ID on
/// `users.stripe_customer_id` and tags the customer with
/// `metadata.scholia_user_id` for cross-system traceability.
async fn get_or_create_stripe_customer(
    state: &AppState,
    user: &AuthUser,
) -> Result<String, AppError> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT stripe_customer_id FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(&state.pool)
            .await
            .map_err(AppError::from)?;

    if let Some(id) = existing {
        return Ok(id);
    }

    let customer = CreateCustomer::new()
        .email(user.email.clone())
        .name(user.display_name.clone())
        .metadata([(String::from("scholia_user_id"), user.id.to_string())])
        .send(&state.stripe)
        .await
        .map_err(|e| {
            tracing::error!("stripe customer.create failed: {e}");
            AppError::Internal("stripe customer create failed".into())
        })?;

    let customer_id = customer.id.to_string();

    sqlx::query("UPDATE users SET stripe_customer_id = $1, updated_at = now() WHERE id = $2")
        .bind(&customer_id)
        .bind(user.id)
        .execute(&state.pool)
        .await
        .map_err(AppError::from)?;

    Ok(customer_id)
}

/// Upsert the local subscription row from a freshly-fetched Stripe
/// Subscription, sync `user_roles` (only the three paid roles, never
/// the `honorary` comp role), and record the event ID for dedup —
/// all in one transaction.
async fn apply_subscription(
    pool: &PgPool,
    config: &AppConfig,
    event_id: &str,
    sub: &stripe_shared::Subscription,
) -> Result<(), AppError> {
    let customer_id = sub.customer.id().to_string();

    // Resolve to our user_id.
    let user_id = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE stripe_customer_id = $1",
    )
    .bind(&customer_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?
    {
        Some(id) => id,
        None => {
            // Webhook fired for a customer we don't know about (e.g.
            // orphan from a failed registration write). Try the
            // metadata fallback before giving up.
            let from_meta = sub
                .metadata
                .get("scholia_user_id")
                .and_then(|s| Uuid::parse_str(s).ok());
            match from_meta {
                Some(id) => id,
                None => {
                    // Permanent failure: customer has no DB mapping
                    // AND no scholia_user_id metadata. In production
                    // this should never happen (every real customer
                    // is created via get_or_create_stripe_customer
                    // which writes both). Log as error so monitoring
                    // catches it; ack 200 anyway because retrying
                    // can't fix it. Common cases:
                    //   - `stripe trigger` test fixtures (dev only)
                    //   - sub manually created in Stripe Dashboard
                    //   - bug in our customer-create flow lost metadata
                    tracing::error!(
                        "webhook for unknown stripe customer {customer_id} (sub {}); no DB mapping or metadata — acking but not processing",
                        sub.id
                    );
                    return Ok(());
                }
            }
        }
    };

    // Pick the first subscription item's price (single-price subs only
    // for now — all three of our tiers are single-item). `current_period_end`
    // moved from the subscription to per-item in the 2024 API.
    //
    // Empty items.data is a permanent shape we can't process; ack 200
    // (no retry will fix it) but log as error for ops visibility.
    let item = match sub.items.data.first() {
        Some(i) => i,
        None => {
            tracing::error!(
                "subscription {} has no items; acking but not processing",
                sub.id
            );
            return Ok(());
        }
    };
    let price_id = item.price.id.to_string();

    let tier_label = match config.tier_label_for_price_id(&price_id) {
        Some(t) => t,
        None => {
            // Unknown price — log and ack. Don't try to assign a role.
            tracing::warn!(
                "subscription {} on unknown price {price_id}; treating as inactive",
                sub.id
            );
            "unknown"
        }
    };

    let status = sub.status.as_str().to_string();
    let period_end: i64 = item.current_period_end;

    let mut tx = pool.begin().await.map_err(AppError::from)?;

    // Dedup gate. If this event ID has already been processed, skip.
    let inserted = sqlx::query(
        "INSERT INTO stripe_processed_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING",
    )
    .bind(event_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    if inserted.rows_affected() == 0 {
        // Already processed; commit (no-op) and return.
        tx.commit().await.map_err(AppError::from)?;
        return Ok(());
    }

    // Upsert subscription row. Stripe's subscription_id is unique.
    sqlx::query(
        "INSERT INTO subscriptions (
            user_id, stripe_subscription_id, stripe_price_id, tier,
            status, current_period_end, cancel_at_period_end
        ) VALUES ($1, $2, $3, $4, $5, to_timestamp($6), $7)
        ON CONFLICT (stripe_subscription_id) DO UPDATE SET
            stripe_price_id = EXCLUDED.stripe_price_id,
            tier = EXCLUDED.tier,
            status = EXCLUDED.status,
            current_period_end = EXCLUDED.current_period_end,
            cancel_at_period_end = EXCLUDED.cancel_at_period_end,
            updated_at = now()",
    )
    .bind(user_id)
    .bind(sub.id.to_string())
    .bind(&price_id)
    .bind(tier_label)
    .bind(&status)
    .bind(period_end)
    .bind(sub.cancel_at_period_end)
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // Sync paid roles. Drop all three first, then add the right one
    // if the sub is access-granting. Touches only paid roles — the
    // honorary comp role is untouched by design.
    sqlx::query(
        "DELETE FROM user_roles
         WHERE user_id = $1
           AND role_id IN (
             SELECT id FROM roles
             WHERE name IN ('scholiast', 'scholiast_benefactor', 'scholiast_patron')
           )",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // Statuses that grant access: active, trialing, past_due (grace).
    let grants_access = matches!(status.as_str(), "active" | "trialing" | "past_due");

    if grants_access {
        if let Some(role_name) = config.role_for_price_id(&price_id) {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id)
                 SELECT $1, id FROM roles WHERE name = $2
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(role_name)
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
        }
    }

    tx.commit().await.map_err(AppError::from)?;

    Ok(())
}

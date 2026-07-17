use std::net::SocketAddr;

use api::system::config::AppConfig;
use api::system::state::AppState;
use axum::http::HeaderValue;
use sqlx::PgPool;
use time::Duration;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::CorsLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // Default filter silences two noisy modules from async-stripe that
    // emit WARN spans whenever Stripe's API version drifts ahead of the
    // SDK codegen. The spans include the entire raw event payload as
    // context, so each warn dumps a wall of text. Subscription events
    // still parse correctly; the warnings are informational. Override
    // via RUST_LOG=stripe_webhook=warn if you need to investigate.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,stripe_webhook=error,stripe_shared::api_version=error"
                    .parse()
                    .unwrap()
            }),
        )
        .init();

    // Subcommand dispatch. The init container in cluster runs
    // `api migrate` to apply embedded sqlx migrations before the main
    // server boots; the same binary serves both modes.
    if std::env::args().nth(1).as_deref() == Some("migrate") {
        tracing::info!("Running migrations…");
        api::system::migrate::run(
            dataduct::db::pg_connect_options(None).expect("Invalid Postgres connection config"),
        )
        .await
        .expect("Migrations failed");
        tracing::info!("Migrations applied.");
        return;
    }

    let config = AppConfig::from_env();

    let pool = PgPool::connect_with(
        dataduct::db::pg_connect_options(None).expect("Invalid Postgres connection config"),
    )
    .await
    .expect("Failed to connect to database");

    let session_store = PostgresStore::new(pool.clone());

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("scholia_session")
        .with_secure(config.cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::days(30)));

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .frontend_url
                .parse::<HeaderValue>()
                .expect("Invalid FRONTEND_URL for CORS"),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    let stripe_client = stripe::Client::new(config.stripe_api_key.clone());

    let app_state = AppState {
        pool: pool.clone(),
        config,
        stripe: stripe_client,
        purge_client: api::system::cache::build_client(),
    };

    // The full documented surface is assembled in `api::api_router`
    // (shared with the `openapi` bin). Here we split it into an axum
    // router + the OpenAPI doc, then layer on sessions/CORS/state and
    // attach the undocumented Stripe webhook.
    let (router, api) = api::api_router().split_for_parts();

    // Stripe webhook lives on a separate router so it bypasses session +
    // CORS layers (Stripe-Signature is the only auth). Owned by the billing
    // domain; merged here, before with_state.
    let app = router
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", api))
        .layer(session_layer)
        .layer(cors)
        .merge(api::modules::billing::webhook_routes())
        // Sitemaps: undocumented GET-only XML routes at the site root,
        // proxied+cached by nginx. Session/CORS layers are irrelevant.
        .merge(api::modules::seo::routes())
        .with_state(app_state)
        // Outermost: turn any handler panic into a 500 instead of a dropped
        // connection. Wraps every route, including the merged Stripe webhook.
        .layer(CatchPanicLayer::new());

    let addr = "0.0.0.0:4000";
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

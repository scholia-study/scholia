use std::net::SocketAddr;
use std::sync::Arc;

use api::ApiDoc;
use api::config::AppConfig;
use api::state::AppState;
use axum::http::HeaderValue;
use sqlx::PgPool;
use time::Duration;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::cors::CorsLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions_sqlx_store::PostgresStore;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
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

    let config = AppConfig::from_env();

    let pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Session store
    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("scholia_session")
        .with_secure(false) // false for localhost; set true in production
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::days(30)));

    // CORS
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

    // Rate limiting for auth endpoints: 10 requests per 60 seconds per IP
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(6)
            .burst_size(10)
            .finish()
            .expect("Failed to build governor config"),
    );
    let rate_limit_layer = GovernorLayer::new(governor_config);

    let stripe_client = stripe::Client::new(config.stripe_api_key.clone());

    let app_state = AppState {
        pool: pool.clone(),
        config,
        stripe: stripe_client,
        purge_client: api::cache::build_client(),
    };

    // Auth routes (rate limited)
    let auth_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(api::handlers::auth::register))
        .routes(utoipa_axum::routes!(api::handlers::auth::login))
        .routes(utoipa_axum::routes!(api::handlers::auth::logout))
        .routes(utoipa_axum::routes!(api::handlers::auth::me))
        .routes(utoipa_axum::routes!(api::handlers::auth::forgot_password))
        .routes(utoipa_axum::routes!(api::handlers::auth::reset_password))
        .routes(utoipa_axum::routes!(api::handlers::auth::verify_email))
        .routes(utoipa_axum::routes!(api::handlers::github::github_login))
        .routes(utoipa_axum::routes!(api::handlers::github::github_callback))
        .layer(rate_limit_layer);

    // Authenticated routes (no rate limiting needed)
    let user_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            api::handlers::auth::get_profile,
            api::handlers::auth::update_profile
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::auth::request_password_change
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_quotations,
            api::handlers::quotations::create_quotation
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::delete_quotation
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_notes,
            api::handlers::quotations::create_note
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::update_note,
            api::handlers::quotations::delete_note
        ))
        .routes(utoipa_axum::routes!(api::handlers::quotations::list_tags))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_all_quotations
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_all_notes
        ))
        // Article endpoints
        .routes(utoipa_axum::routes!(
            api::handlers::articles::create_article
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::list_user_articles
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::get_user_article,
            api::handlers::articles::update_article
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::publish_article
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::archive_article
        ))
        // Article quotation endpoints
        .routes(utoipa_axum::routes!(
            api::handlers::article_quotations::create_article_quotation,
            api::handlers::article_quotations::list_article_quotations
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::article_quotations::delete_article_quotation
        ))
        // Feedback (user-submit). Admin endpoints live in admin_router.
        .routes(utoipa_axum::routes!(
            api::handlers::feedback::create_feedback
        ))
        // Billing — Stripe Embedded Checkout + Customer Portal.
        // Webhook intentionally lives outside this router (public,
        // signature-verified, no session/CORS).
        .routes(utoipa_axum::routes!(
            api::handlers::billing::create_checkout_session
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::billing::create_portal_session
        ));

    // Admin routes — gated per-handler via Permission::AdminPanel.
    // Note: the `articles/{slug}/labels` endpoints live here for URL
    // consistency with other admin routes, but they're gated by
    // Permission::ArticleLabelsManage, not AdminPanel — editors qualify.
    let admin_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(api::handlers::feedback::list_feedback))
        .routes(utoipa_axum::routes!(
            api::handlers::feedback::get_feedback,
            api::handlers::feedback::update_feedback
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::apply_article_label
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::remove_article_label
        ));

    // Public routes (no rate limiting)
    let public_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(api::handlers::books::list_books))
        .routes(utoipa_axum::routes!(api::handlers::books::get_book))
        .routes(utoipa_axum::routes!(api::handlers::books::get_book_about))
        .routes(utoipa_axum::routes!(api::handlers::library::get_library))
        .routes(utoipa_axum::routes!(api::handlers::toc::get_toc))
        .routes(utoipa_axum::routes!(api::handlers::nodes::get_node))
        .routes(utoipa_axum::routes!(api::handlers::page::get_node_page))
        .routes(utoipa_axum::routes!(
            api::handlers::resources::list_resources
        ))
        // Public article endpoints
        .routes(utoipa_axum::routes!(
            api::handlers::articles::list_published_articles
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::get_published_article
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::get_article_by_id
        ))
        .routes(utoipa_axum::routes!(api::handlers::articles::list_topics))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::list_editorial_labels
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::articles::batch_sentences
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::article_quotations::get_article_quotation
        ))
        // Public user profiles + by-id redirect resolver.
        .routes(utoipa_axum::routes!(
            api::handlers::users::get_public_profile
        ))
        .routes(utoipa_axum::routes!(api::handlers::users::get_handle_by_id));

    // Editor routes (auth checked in each handler)
    let editor_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            api::handlers::resources::create_resource
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::resources::update_resource,
            api::handlers::resources::delete_resource
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::sources::search_sources,
            api::handlers::sources::create_source
        ))
        .routes(utoipa_axum::routes!(api::handlers::sources::browse_sources))
        .routes(utoipa_axum::routes!(
            api::handlers::sources::get_source,
            api::handlers::sources::update_source,
            api::handlers::sources::delete_source
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::sources::add_source_person
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::sources::remove_source_person
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::sources::check_source_references
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::persons::search_persons,
            api::handlers::persons::create_person
        ))
        .routes(utoipa_axum::routes!(api::handlers::persons::update_person));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(auth_router)
        .merge(user_router)
        .merge(public_router)
        .merge(editor_router)
        .merge(admin_router)
        .split_for_parts();

    // Stripe webhook lives on a separate router so it bypasses
    // session + CORS layers. Stripe-signature header is the only auth.
    let webhook_router = axum::Router::new().route(
        "/api/webhooks/stripe",
        axum::routing::post(api::handlers::billing::stripe_webhook),
    );

    let app = router
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", api))
        .layer(session_layer)
        .layer(cors)
        .merge(webhook_router)
        .with_state(app_state);

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

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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
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

    let app_state = AppState {
        pool: pool.clone(),
        config,
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
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_tags
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_all_quotations
        ))
        .routes(utoipa_axum::routes!(
            api::handlers::quotations::list_all_notes
        ));

    // Public routes (no rate limiting)
    let public_router = OpenApiRouter::new()
        .routes(utoipa_axum::routes!(api::handlers::books::list_books))
        .routes(utoipa_axum::routes!(api::handlers::books::get_book))
        .routes(utoipa_axum::routes!(api::handlers::toc::get_toc))
        .routes(utoipa_axum::routes!(api::handlers::nodes::get_node))
        .routes(utoipa_axum::routes!(api::handlers::page::get_node_page))
        .routes(utoipa_axum::routes!(api::handlers::resources::list_resources));

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
        .routes(utoipa_axum::routes!(
            api::handlers::sources::update_source
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
        .routes(utoipa_axum::routes!(
            api::handlers::persons::update_person
        ));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(auth_router)
        .merge(user_router)
        .merge(public_router)
        .merge(editor_router)
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", api))
        .layer(session_layer)
        .layer(cors)
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

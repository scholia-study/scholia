use std::env;

use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

mod db;
mod error;
mod handlers;
mod models;

#[derive(OpenApi)]
#[openapi(
    info(title = "Prospero API", version = "0.1.0"),
    components(schemas(
        models::book::BookSummary,
        models::book::BookDetail,
        models::toc::TocNodeResponse,
        models::node::NodeDetail,
        models::node::ContentBlockResponse,
        models::node::SentenceResponse,
        models::page::NodePage,
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(handlers::books::list_books))
        .routes(utoipa_axum::routes!(handlers::books::get_book))
        .routes(utoipa_axum::routes!(handlers::toc::get_toc))
        .routes(utoipa_axum::routes!(handlers::nodes::get_node))
        .routes(utoipa_axum::routes!(handlers::page::get_node_page))
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", api))
        .layer(cors)
        .with_state(pool);

    let addr = "0.0.0.0:4000";
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

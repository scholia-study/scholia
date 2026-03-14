use utoipa::OpenApi;

pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

#[derive(OpenApi)]
#[openapi(
    info(title = "Prospero API", version = "0.1.0"),
    paths(
        handlers::books::list_books,
        handlers::books::get_book,
        handlers::toc::get_toc,
        handlers::nodes::get_node,
        handlers::page::get_node_page,
    ),
    components(schemas(
        models::book::BookSummary,
        models::book::BookDetail,
        models::toc::TocNodeResponse,
        models::node::NodeDetail,
        models::node::ContentBlockResponse,
        models::node::SentenceResponse,
        models::node::PageMarkerResponse,
        models::node::FootnoteResponse,
        models::node::FootnoteSentenceResponse,
        models::page::NodePage,
    ))
)]
pub struct ApiDoc;

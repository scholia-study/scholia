use utoipa::OpenApi;

pub mod auth;
pub mod config;
pub mod db;
pub mod email;
pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    info(title = "Prospero API", version = "0.1.0"),
    paths(
        handlers::books::list_books,
        handlers::books::get_book,
        handlers::toc::get_toc,
        handlers::nodes::get_node,
        handlers::page::get_node_page,
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::logout,
        handlers::auth::me,
        handlers::auth::forgot_password,
        handlers::auth::reset_password,
        handlers::auth::verify_email,
        handlers::auth::get_profile,
        handlers::auth::update_profile,
        handlers::auth::request_password_change,
        handlers::github::github_login,
        handlers::github::github_callback,
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
        handlers::auth::RegisterRequest,
        handlers::auth::LoginRequest,
        handlers::auth::ForgotPasswordRequest,
        handlers::auth::ResetPasswordRequest,
        handlers::auth::AuthResponse,
        handlers::auth::MessageResponse,
        handlers::auth::ProfileResponse,
        handlers::auth::LinkedProvider,
        handlers::auth::UpdateProfileRequest,
    ))
)]
pub struct ApiDoc;

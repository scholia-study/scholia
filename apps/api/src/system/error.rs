use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, message).into_response()
    }
}

/// Map a missing row to a caller-chosen `AppError`, while letting real
/// database failures (connection loss, pool timeout, …) surface as `Internal`
/// — which is logged — instead of masquerading as a 404/400 with no trace.
/// Replaces `.on_missing(|| AppError::NotFound(...))`, which swallows every error.
pub trait SqlxResultExt<T> {
    fn on_missing(self, f: impl FnOnce() -> AppError) -> Result<T, AppError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn on_missing(self, f: impl FnOnce() -> AppError) -> Result<T, AppError> {
        self.map_err(|e| match e {
            sqlx::Error::RowNotFound => f(),
            other => AppError::from(other),
        })
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = err {
            return AppError::NotFound("Not found".to_string());
        }
        // Map the constraint/format violations a bad request can trigger to
        // 4xx instead of a blanket 500. Messages stay generic — the raw DB
        // error (constraint/column names) is logged at debug, never returned.
        if let sqlx::Error::Database(db) = &err {
            if let Some(code) = db.code() {
                tracing::debug!("database error {code}: {db}");
                return match code.as_ref() {
                    "23505" => AppError::Conflict("That already exists.".to_string()),
                    "23503" => {
                        AppError::BadRequest("References a record that doesn't exist.".to_string())
                    }
                    "23514" => {
                        AppError::BadRequest("A value is outside the allowed range.".to_string())
                    }
                    "22P02" | "22007" | "22008" => {
                        AppError::BadRequest("A value has an invalid format.".to_string())
                    }
                    _ => AppError::Internal(err.to_string()),
                };
            }
        }
        AppError::Internal(err.to_string())
    }
}

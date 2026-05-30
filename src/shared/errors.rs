use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound(&'static str),
    #[error("conflict")]
    Conflict(&'static str, &'static str),
    #[error("bad request")]
    BadRequest(&'static str, &'static str),
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Invalid or missing API key"),
            AppError::NotFound(code) => (StatusCode::NOT_FOUND, code, "Resource not found"),
            AppError::Conflict(code, message) => (StatusCode::CONFLICT, code, message),
            AppError::BadRequest(code, message) => (StatusCode::BAD_REQUEST, code, message),
            AppError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error", "Database operation failed"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Internal server error"),
        };
        (status, Json(ErrorEnvelope { error: ErrorBody { code, message } })).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

pub type TinyBoardsResult<T> = Result<T, TinyBoardsError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum TinyBoardsError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error")]
    Validation(Vec<String>),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    error_code: u16,
}

impl TinyBoardsError {
    /// Convenience for migration: maps a numeric status code + message to the appropriate variant.
    /// Prefer using typed variants directly in new code.
    pub fn from_message(status: u16, msg: &str) -> Self {
        match status {
            400 => TinyBoardsError::BadRequest(msg.to_string()),
            401 => TinyBoardsError::Unauthorized,
            403 => TinyBoardsError::Forbidden(msg.to_string()),
            404 => TinyBoardsError::NotFound(msg.to_string()),
            409 => TinyBoardsError::Conflict(msg.to_string()),
            422 => TinyBoardsError::Validation(vec![msg.to_string()]),
            429 => TinyBoardsError::RateLimited,
            _ => TinyBoardsError::Internal(msg.to_string()),
        }
    }

    /// Convenience for migration: wraps an underlying error with a message.
    /// Prefer using typed variants directly in new code.
    pub fn from_error_message<E: std::fmt::Debug>(error: E, status: u16, msg: &str) -> Self {
        tracing::error!("{}: {:?}", msg, error);
        Self::from_message(status, msg)
    }
}

impl actix_web::error::ResponseError for TinyBoardsError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            Self::Validation(errors) => {
                #[derive(Serialize)]
                struct ValidationErrorResponse {
                    error: String,
                    error_code: u16,
                    details: Vec<String>,
                }
                HttpResponse::build(status).json(ValidationErrorResponse {
                    error: "Validation error".to_string(),
                    error_code: status.as_u16(),
                    details: errors.clone(),
                })
            }
            _ => {
                HttpResponse::build(status).json(ErrorResponse {
                    error: self.to_string(),
                    error_code: status.as_u16(),
                })
            }
        }
    }
}

impl From<diesel::result::Error> for TinyBoardsError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => {
                TinyBoardsError::NotFound("Record not found".to_string())
            }
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ) => TinyBoardsError::Conflict(info.message().to_string()),
            other => TinyBoardsError::Database(other.to_string()),
        }
    }
}

impl From<bb8::RunError<diesel_async::pooled_connection::PoolError>> for TinyBoardsError {
    fn from(e: bb8::RunError<diesel_async::pooled_connection::PoolError>) -> Self {
        TinyBoardsError::Database(format!("Connection pool error: {}", e))
    }
}

impl From<serde_json::Error> for TinyBoardsError {
    fn from(e: serde_json::Error) -> Self {
        TinyBoardsError::Internal(format!("JSON error: {}", e))
    }
}

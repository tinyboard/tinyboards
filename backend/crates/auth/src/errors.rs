use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

/// Auth-specific error type with HTTP status codes and user-facing messages.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account is banned")]
    AccountBanned,

    #[error("Account is deleted")]
    AccountDeleted,

    #[error("Account application is pending")]
    ApplicationPending,

    #[error("Registration is closed")]
    RegistrationClosed,

    #[error("Invite code required")]
    InviteRequired,

    #[error("Invalid invite code")]
    InvalidInvite,

    #[error("Application answer required")]
    ApplicationRequired,

    #[error("Email required for this registration mode")]
    EmailRequired,

    #[error("Invalid username: must be 1-30 chars, starting with a letter, containing only letters, numbers, and underscores")]
    InvalidUsername,

    #[error("Username is not allowed")]
    UsernameFiltered,

    #[error("Password must be between 10-60 characters")]
    InvalidPasswordLength,

    #[error("A user with that {0} already exists")]
    DuplicateUser(String),

    #[error("Invalid or expired access token")]
    InvalidAccessToken,

    #[error("Invalid or expired refresh token")]
    InvalidRefreshToken,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid or expired reset token")]
    InvalidResetToken,

    #[error("Reset token already used")]
    ResetTokenUsed,

    #[error("Invalid verification token")]
    InvalidVerificationToken,

    #[error("Already verified")]
    AlreadyVerified,

    #[error("Already logged in")]
    AlreadyLoggedIn,

    #[error("Login required")]
    LoginRequired,

    #[error("Incorrect current password")]
    IncorrectPassword,

    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Missing or invalid X-Requested-With header")]
    CsrfViolation,
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::AccountBanned => StatusCode::FORBIDDEN,
            Self::AccountDeleted => StatusCode::UNAUTHORIZED,
            Self::ApplicationPending => StatusCode::FORBIDDEN,
            Self::RegistrationClosed => StatusCode::FORBIDDEN,
            Self::InviteRequired => StatusCode::FORBIDDEN,
            Self::InvalidInvite => StatusCode::FORBIDDEN,
            Self::ApplicationRequired => StatusCode::BAD_REQUEST,
            Self::EmailRequired => StatusCode::BAD_REQUEST,
            Self::InvalidUsername => StatusCode::BAD_REQUEST,
            Self::UsernameFiltered => StatusCode::BAD_REQUEST,
            Self::InvalidPasswordLength => StatusCode::BAD_REQUEST,
            Self::DuplicateUser(_) => StatusCode::CONFLICT,
            Self::InvalidAccessToken => StatusCode::UNAUTHORIZED,
            Self::InvalidRefreshToken => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::InvalidResetToken => StatusCode::BAD_REQUEST,
            Self::ResetTokenUsed => StatusCode::BAD_REQUEST,
            Self::InvalidVerificationToken => StatusCode::BAD_REQUEST,
            Self::AlreadyVerified => StatusCode::BAD_REQUEST,
            Self::AlreadyLoggedIn => StatusCode::BAD_REQUEST,
            Self::LoginRequired => StatusCode::UNAUTHORIZED,
            Self::IncorrectPassword => StatusCode::UNAUTHORIZED,
            Self::HashingFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TokenGenerationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CsrfViolation => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    error_code: u16,
}

impl actix_web::error::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        AuthError::status_code(self)
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(ErrorResponse {
            error: self.to_string(),
            error_code: status.as_u16(),
        })
    }
}

impl From<diesel::result::Error> for AuthError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => AuthError::InvalidCredentials,
            other => AuthError::DatabaseError(other.to_string()),
        }
    }
}

impl From<bb8::RunError<diesel_async::pooled_connection::PoolError>> for AuthError {
    fn from(e: bb8::RunError<diesel_async::pooled_connection::PoolError>) -> Self {
        AuthError::DatabaseError(format!("Connection pool error: {}", e))
    }
}

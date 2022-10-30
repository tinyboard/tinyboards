use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::Error;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse<'a> {
    code: u16,
    error: &'a str,
}

#[derive(Debug, Error)]
pub struct TinyBoardsError {
    pub message: String,
    pub error_code: u16,
}

impl TinyBoardsError {
    pub fn new(error_code: u16, message: String) -> Self {
        Self {
            message,
            error_code,
        }
    }

    pub fn from_string(message: &str, error_code: u16) -> Self {
        Self::new(error_code, String::from(message))
    }

    pub fn err_500() -> Self {
        Self::new(500, String::from("Internal Server Error"))
    }

    pub fn err_401() -> Self {
        Self::new(401, String::from("You must be logged in to do that!"))
    }
}

impl std::fmt::Display for TinyBoardsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error {}: {}", self.error_code, self.message)
    }
}

impl error::ResponseError for TinyBoardsError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            code: self.error_code,
            error: &self.message,
        })
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.error_code).expect("Invalid error code")
    }
}
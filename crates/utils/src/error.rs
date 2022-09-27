use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::Error;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse<'a> {
    code: u16,
    error: &'a str,
}

#[derive(Debug, Error)]
pub struct PorplError {
    pub message: String,
    pub error_code: u16,
}

impl PorplError {
    pub fn new(error_code: u16, message: String) -> Self {
        Self {
            message,
            error_code,
        }
    }
}

impl std::fmt::Display for PorplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error {}: {}", self.error_code, self.message)
    }
}

impl error::ResponseError for PorplError {
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

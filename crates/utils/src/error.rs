use std::fmt;
use std::fmt::{Debug, Display};
use tracing_error::SpanTrace;

#[derive(serde::Serialize)]
struct ApiError {
  error: String,
}

pub struct TinyBoardsError {
    pub message: Option<String>,
    pub inner: anyhow::Error,
    pub context: SpanTrace,
}

impl TinyBoardsError {
    /// Create a TinyBoardsError from a message, including stack trace
    pub fn from_message(message: &str) -> Self {
        let inner = anyhow::anyhow!("{}", message);
        TinyBoardsError { 
            message: Some(message.into()), 
            inner, 
            context: SpanTrace::capture(), 
        }
    }

    /// Create a TinyBoardsError from a error and a message, including stack trace
    pub fn from_error_message<E>(error: E, message: &str) -> Self 
    where 
        E: Into<anyhow::Error>, 
    {
        TinyBoardsError { 
            message: Some(message.into()), 
            inner: error.into(), 
            context: SpanTrace::capture(),
         }
    }

    /// Add a message to existing error (or overwrite error)
    pub fn with_message(self, message: &str) -> Self {
        TinyBoardsError {
            message: Some(message.into()),
            ..self
        }
    }

    pub fn to_json(&self) -> Result<String, Self> {
        let api_error = match &self.message {
          Some(error) => ApiError {
            error: error.into(),
          },
          None => ApiError {
            error: "Unknown".into(),
          },
        };
    
        Ok(serde_json::to_string(&api_error)?)
    }
}

impl<T> From<T> for TinyBoardsError
where
    T: Into<anyhow::Error> 
{
    fn from(t: T) -> Self {
        TinyBoardsError {
            message: None,
            inner: t.into(),
            context: SpanTrace::capture(),
        }
    }
}

impl Debug for TinyBoardsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TinyBoardsError")
            .field("message", &self.message)
            .field("inner", &self.inner)
            .field("context", &"SpanTrace")
            .finish()
    }
}

impl Display for TinyBoardsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(message) = &self.message {
            write!(f, "{}: ", message)?;
        }
        writeln!(f, "{}", self.inner)?;
        fmt::Display::fmt(&self.context, f)
    }
}

impl actix_web::error::ResponseError for TinyBoardsError {
    fn status_code(&self) -> http::StatusCode {
        match self.inner.downcast_ref::<diesel::result::Error>() {
            Some(diesel::result::Error::NotFound) => http::StatusCode::NOT_FOUND,
            _ => http::StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        if let Some(message) = &self.message {
            actix_web::HttpResponse::build(self.status_code()).json(ApiError {
                error: message.into(),
            })
        } else {
            actix_web::HttpResponse::build(self.status_code())
                .content_type("text/plain")
                .body(self.inner.to_string())
        }
    }
}
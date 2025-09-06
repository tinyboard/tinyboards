// Webfinger functionality disabled for local-only operation
use actix_web::{web, HttpResponse};
use tinyboards_utils::error::TinyBoardsError;
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
    resource: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        ".well-known/webfinger",
        web::get().to(get_webfinger_response)
    );
}

// Webfinger disabled for local-only operation  
async fn get_webfinger_response() -> Result<HttpResponse, TinyBoardsError> {
    Ok(HttpResponse::NotFound().json(serde_json::json!({
        "error": "Webfinger not supported in local-only mode"
    })))
}
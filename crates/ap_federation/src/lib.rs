pub mod activity_queue;
#[cfg(feature = "actix-web")]
pub mod actix_web;
#[cfg(feature = "axum")]
pub mod axum;
pub mod config;
pub mod error;
pub mod fetch;
pub mod http_signatures;
pub mod protocol;
pub(crate) mod reqwest_shim;
pub mod traits;

pub use activitystreams_kinds as kinds;

/// Mime type for Activitypub data, used for `Accept` and `Content-Type` HTTP headers
pub static FEDERATION_CONTENT_TYPE: &str = "application/activity+json";

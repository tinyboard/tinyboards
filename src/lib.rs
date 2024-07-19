#![recursion_limit = "512"]
pub mod api_routes;
pub mod code_migrations;
pub mod root_span_builder;
pub mod scheduled_tasks;
#[cfg(feature = "console")]
pub mod telemetry;

use tinyboards_utils::error::TinyBoardsError;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};
use url::Url;
use async_graphql::*;

pub fn init_logging(opentelemetry_url: &Option<Url>) -> Result<(), TinyBoardsError> {
    LogTracer::init()
        .map_err(|_| TinyBoardsError::from_message(500, "failed to initialize log tracer"))?;

    let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let targets = log_description
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .map_err(|_| TinyBoardsError::from_message(500, "failed to get logger targets"))?;

    let format_layer = tracing_subscriber::fmt::layer().with_filter(targets.clone());

    let subscriber = Registry::default()
        .with(format_layer)
        .with(ErrorLayer::default());

    if let Some(_url) = opentelemetry_url {
        #[cfg(feature = "console")]
        telemetry::init_tracing(_url.as_ref(), subscriber, targets)?;
        #[cfg(not(feature = "console"))]
        tracing::error!("Feature `console` must be enabled for opentelemetry tracing");
    } else {
        set_global_default(subscriber).map_err(|_| {
            TinyBoardsError::from_message(500, "failed to set global default for logger")
        })?;
    }

    Ok(())
}

struct Query;


/* This would be the master query object, we would need to define all query operations here I believe */
// #[Object]
// impl Query {


// EXAMPLE
//  async fn users(&self, username: String) -> Result<Option<User>> {
//      some query logic in here I suppose    
//}

// }

#![recursion_limit = "512"]
pub mod api_routes;
pub mod root_span_builder;
pub mod scheduled_tasks;
#[cfg(feature = "console")]
pub mod telemetry;

use porpl_utils::error::PorplError;
use tracing::subscriber::set_global_default;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, Layer, Registry, layer::SubscriberExt};
use url::Url;

pub fn init_logging(opentelemetry_url: &Option<Url>) -> Result<(), PorplError> {
    LogTracer::init()
        .map_err(|_| PorplError::from_string("failed to initialize log tracer", 500))?;

    let log_description = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

    let targets = log_description
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .map_err(|_| PorplError::from_string("failed to get logger targets", 500))?;
    
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
        set_global_default(subscriber)
            .map_err(|_| PorplError::from_string("failed to set global default for logger", 500))?;
    }

    Ok(())
}
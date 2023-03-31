#[macro_use]
extern crate diesel_migrations;

use actix_web::{web::Data, *};
use diesel_migrations::EmbeddedMigrations;
use dotenv::dotenv;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{thread, time::Duration};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    request::build_user_agent,
    utils::{get_rate_limit_config},
};
use tinyboards_db::{models::secret::Secret, utils::{build_db_pool, run_migrations, get_db_url}};
use tinyboards_server::{
    api_routes, code_migrations::run_advanced_migrations, init_logging,
    root_span_builder::QuieterRootSpanBuilder, scheduled_tasks,
};
use tinyboards_utils::{error::TinyBoardsError, rate_limit::RateLimitCell, settings::SETTINGS};
use tracing_actix_web::TracingLogger;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// max timeout for http requests
pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[actix_web::main]
async fn main() -> Result<(), TinyBoardsError> {
    dotenv().ok();

    let settings = SETTINGS.to_owned();
    
    // Set up the bb8 connection pool
    let db_url = get_db_url(Some(&settings));
    run_migrations(&db_url);

    init_logging(&settings.opentelemetry_url)
        .map_err(|_| TinyBoardsError::from_message(500, "failed to initialize logger"))?;

    
    let pool = build_db_pool(&settings).await?;

    let _protocol_and_hostname = settings.get_protocol_and_hostname();

    // run advanced migrations
    run_advanced_migrations(&pool, &settings).await?;

    let db_url = get_db_url(Some(&settings));
    thread::spawn(move || {
        scheduled_tasks::setup(db_url).expect("Couldn't setup scheduled tasks");
    });

    let rate_limit_config = get_rate_limit_config(&settings.rate_limit.to_owned().unwrap());
    let rate_limit_cell = RateLimitCell::new(rate_limit_config).await;

    // init the secret
    let db_url = get_db_url(Some(&settings));
    let secret = Secret::init(db_url).expect("Couldn't initialize secrets.");

    println!(
        "Starting http server at {}:{}",
        settings.bind, settings.port
    );

    let reqwest_client = Client::builder()
        .user_agent(build_user_agent(&settings))
        .timeout(REQWEST_TIMEOUT)
        .build()
        .map_err(|_| TinyBoardsError::from_message(500, "could not build reqwest client"))?;

    let retry_policy = ExponentialBackoff {
        max_n_retries: 3,
        max_retry_interval: REQWEST_TIMEOUT,
        min_retry_interval: Duration::from_millis(100),
        backoff_exponent: 2,
    };

    let client: ClientWithMiddleware = ClientBuilder::new(reqwest_client.clone())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let pictrs_client: ClientWithMiddleware = ClientBuilder::new(reqwest_client.clone())
        .with(TracingMiddleware::default())
        .build();

    let settings_bind = settings.clone();
    HttpServer::new(move || {
        let context = TinyBoardsContext::create(
            pool.clone(),
            client.clone(),
            settings.clone(),
            secret.clone(),
            rate_limit_cell.clone(),
        );
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
            .app_data(Data::new(context))
            .app_data(Data::new(rate_limit_cell.clone()))
            .configure(|cfg| api_routes::config(cfg, &rate_limit_cell))
    })
    .bind((settings_bind.bind, settings_bind.port))
    .map_err(|_| TinyBoardsError::from_message(500, "could not bind to ip"))?
    .run()
    .await
    .map_err(|_| TinyBoardsError::from_message(500, "could not start web server"))?;

    Ok(())
}

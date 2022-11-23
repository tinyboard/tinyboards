#[macro_use]
extern crate diesel_migrations;

use crate::diesel_migrations::MigrationHarness;
#[allow(unused_imports)]
use actix::prelude::*;
use actix_files as fs;
use actix_web::{web::Data, *};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::EmbeddedMigrations;
use dotenv::dotenv;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{
    thread,
    time::Duration,
};
use tinyboards_api_common::{data::TinyBoardsContext, request::build_user_agent, utils::{blocking, get_rate_limit_config},};
use tinyboards_db::models::secret::Secret;
use tinyboards_db::utils::get_database_url_from_env;
use tinyboards_server::{
    api_routes, init_logging, root_span_builder::QuieterRootSpanBuilder, scheduled_tasks, media
};
use tinyboards_utils::{
    error::TinyBoardsError,
    rate_limit::RateLimitCell,
    settings::SETTINGS,
};
use tracing_actix_web::TracingLogger;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// max timeout for http requests
pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[actix_web::main]
async fn main() -> Result<(), TinyBoardsError> {
    dotenv().ok();

    let settings = SETTINGS.to_owned();

    init_logging(&settings.opentelemetry_url)
        .map_err(|_| TinyBoardsError::from_message("failed to initialize logger"))?;

    let db_url = match get_database_url_from_env() {
        Ok(url) => url,
        Err(_) => settings.get_database_url(),
    };

    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    let pool = Pool::builder()
        .max_size(settings.database.pool_size)
        .min_idle(Some(1))
        .build(manager)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

    let _protocol_and_hostname = settings.get_protocol_and_hostname();

    blocking(&pool, move |conn| {
        let _ = conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|_| TinyBoardsError::from_message("Couldn't run migrations"))?;
        Ok(()) as Result<(), TinyBoardsError>
    })
    .await??;

    let task_pool = pool.clone();
    thread::spawn(move || {
        scheduled_tasks::setup(task_pool).expect("Couldn't setup scheduled tasks");
    });


    let rate_limit_config =
        get_rate_limit_config(&settings.rate_limit.to_owned().unwrap());
    let rate_limit_cell = RateLimitCell::new(rate_limit_config).await;

    // init the secret
    let conn = &mut pool.get().map_err(|_| {
        TinyBoardsError::from_message(
            "could not establish connection pool for initializing secrets"
        )
    })?;
    let secret = Secret::init(conn).expect("Couldn't initialize secrets.");

    println!(
        "Starting http server at {}:{}",
        settings.bind, settings.port
    );

    let reqwest_client = Client::builder()
        .user_agent(build_user_agent(&settings))
        .timeout(REQWEST_TIMEOUT)
        .build()
        .map_err(|_| TinyBoardsError::from_message("could not build reqwest client"))?;

    let retry_policy = ExponentialBackoff {
        max_n_retries: 3,
        max_retry_interval: REQWEST_TIMEOUT,
        min_retry_interval: Duration::from_millis(100),
        backoff_exponent: 2,
    };

    let client = ClientBuilder::new(reqwest_client.clone())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let pictrs_client = ClientBuilder::new(reqwest_client.clone())
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
            // the extra routes
            .configure(|cfg| media::config(cfg, pictrs_client.clone(), &rate_limit_cell))
            .service(fs::Files::new("/assets", "./assets"))
    })
    .bind((settings_bind.bind, settings_bind.port))
    .map_err(|_| TinyBoardsError::from_message("could not bind to ip"))?
    .run()
    .await
    .map_err(|_| TinyBoardsError::from_message("could not start web server"))?;

    Ok(())
}

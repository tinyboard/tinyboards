#[macro_use]
extern crate diesel_migrations;

use crate::diesel_migrations::MigrationHarness;
use actix::prelude::*;
use actix_web::{web::Data, *};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::EmbeddedMigrations;
use porpl_api_common::{utils::blocking, data::PorplContext, request::build_user_agent};
use porpl_server::{
    api_routes,
    init_logging, 
    scheduled_tasks,
    root_span_builder::QuieterRootSpanBuilder,
};
use porpl_utils::{
    error::PorplError,
    rate_limit::{rate_limiter::RateLimiter, RateLimit},
    settings::{structs::Settings, SETTINGS},
};
use porpl_db::utils::{get_database_url_from_env, DbPool};
use porpl_db::models::secret::Secret;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{
    env,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tracing_actix_web::TracingLogger;
use dotenv::dotenv;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// max timeout for http requests
pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);


#[actix_web::main]
async fn main() -> Result<(), PorplError> {
    dotenv().ok();

    let settings = SETTINGS.to_owned();

    init_logging(&settings.opentelemetry_url)
        .map_err(|_| PorplError::from_string("failed to initialize logger", 500));
    
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
    
    let protocol_and_hostname = settings.get_protocol_and_hostname();

    blocking(&pool, move |conn| {
        let _ = conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|_| PorplError::from_string("Couldn't run migrations", 500))?;
        Ok(()) as Result<(), PorplError>
    })
    .await?;

    let task_pool = pool.clone();
    thread::spawn(move || {
        scheduled_tasks::setup(task_pool).expect("Couldn't setup scheduled tasks");
    });

    let rate_limiter = RateLimit {
        rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
        rate_limit_config: settings.rate_limit.to_owned().unwrap_or_default(),
    };

    // init the secret
    let conn = &mut pool.get()
        .map_err(|_| PorplError::from_string("could not establish connection pool for initializing secrets", 500))?;
    let secret = Secret::init(conn).expect("Couldn't initialize secrets.");

    println!(
        "Starting http server at {}:{}",
        settings.bind, settings.port
    );

    let reqwest_client = Client::builder()
        .user_agent(build_user_agent(&settings))
        .timeout(REQWEST_TIMEOUT)
        .build()
        .map_err(|_| PorplError::from_string("could not build reqwest client", 500))?;

    let settings_bind = settings.clone();
    HttpServer::new(move || {
        let context = PorplContext::create(
            pool,
            client,
            settings,
            secret,
        );
        let rate_limiter = rate_limiter.clone();
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
            .app_data(Data::new(context))
            .app_data(Data::new(rate_limiter.clone()))
            .configure(|cfg| api_routes::config(cfg, &rate_limiter))
    })
    .bind((settings_bind.bind, settings_bind.port))
    .map_err(|_| PorplError::from_string("could not bind to ip", 500))?
    .run()
    .await
    .map_err(|_| PorplError::from_string("could not start web server", 500))?;

    Ok(())
}
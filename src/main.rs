#[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{web::Data, *};
use diesel_migrations::EmbeddedMigrations;
use dotenv::dotenv;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use std::{thread, time::Duration};
use tinyboards_api::{
    context::TinyBoardsContext,
    utils::request::build_user_agent,
};
use tinyboards_api::gen_schema;
use tinyboards_db::{
    models::secret::Secret,
    utils::{build_db_pool, get_db_url, run_migrations},
};
use tinyboards_server::{
    api_routes, code_migrations::run_advanced_migrations, init_logging,
    root_span_builder::QuieterRootSpanBuilder, scheduled_tasks,
};
use tinyboards_utils::utils::ensure_upload_directories;
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};
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

    // ensure upload directories exist
    let media_path = settings.get_media_path();
    ensure_upload_directories(&media_path).await.map_err(|e| {
        TinyBoardsError::from_message(500, &format!("Failed to create upload directories: {}", e))
    })?;

    // Initialize storage backend
    let storage = tinyboards_api::storage::StorageBackend::from_settings(&settings).await
        .map_err(|e| {
            TinyBoardsError::from_message(500, &format!("Failed to initialize storage: {}", e))
        })?;

    tracing::info!("Storage backend initialized: {:?}", storage.backend_type());

    let db_url = get_db_url(Some(&settings));
    thread::spawn(move || {
        scheduled_tasks::setup(db_url).expect("Couldn't setup scheduled tasks");
    });

    // init the secret
    let db_url = get_db_url(Some(&settings));
    let secret = Secret::init(db_url).expect("Couldn't initialize secrets.");

    // make sure local site is setup

    println!(
        "Starting http server at {}:{}",
        settings.bind, settings.port
    );

    let user_agent = build_user_agent(&settings);
    let reqwest_client = Client::builder()
        .user_agent(user_agent.clone())
        .timeout(REQWEST_TIMEOUT)
        .connect_timeout(REQWEST_TIMEOUT)
        .build()?;

    let retry_policy = ExponentialBackoff {
        max_n_retries: 3,
        max_retry_interval: REQWEST_TIMEOUT,
        min_retry_interval: Duration::from_millis(100),
        backoff_exponent: 2,
    };

    let graphql_schema = gen_schema();

    let client: ClientWithMiddleware = ClientBuilder::new(reqwest_client.clone())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let settings_bind = settings.clone();
    HttpServer::new(move || {
        let context = TinyBoardsContext::create(
            pool.clone(),
            client.clone(),
            settings.clone(),
            secret.clone(),
            storage.clone(),
            //rate_limit_cell.clone(),
            graphql_schema.clone(),
        );

        // Configure CORS based on settings
        let mut cors_config = Cors::default();

        // Add allowed origins
        for origin in &settings.cors.allowed_origins {
            cors_config = cors_config.allowed_origin(origin);
        }

        // Configure methods
        let methods: Vec<actix_web::http::Method> = settings.cors.allowed_methods
            .iter()
            .filter_map(|method| method.parse().ok())
            .collect();
        cors_config = cors_config.allowed_methods(methods);

        // Configure headers
        let headers: Vec<actix_web::http::header::HeaderName> = settings.cors.allowed_headers
            .iter()
            .filter_map(|header| header.parse().ok())
            .collect();
        cors_config = cors_config.allowed_headers(headers);

        // Configure credentials and max age
        cors_config = cors_config
            .supports_credentials()
            .max_age(settings.cors.max_age as usize);

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors_config)
            .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
            .app_data(Data::new(context))
            // GraphQL
            .configure(api_routes::graphql_config)
            // Media file serving - always use OpenDAL handler (works for all backends)
            .configure(api_routes::media_files_config)
    })
    .bind((settings_bind.bind, settings_bind.port))
    .map_err(|_| TinyBoardsError::from_message(500, "could not bind to ip"))?
    .run()
    .await
    .map_err(|_| TinyBoardsError::from_message(500, "could not start web server"))?;

    Ok(())
}

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
use tinyboards_api_graphql::{
    context::TinyBoardsContext,
    utils::request::build_user_agent,
};
use tinyboards_api_graphql::gen_schema;
//use tinyboards_api_graphql::config as graphql_config;
//use tinyboards_apub::{VerifyUrlData, FEDERATION_HTTP_FETCH_LIMIT};
use tinyboards_db::{
    models::secret::Secret,
    utils::{build_db_pool, get_db_url, run_migrations},
};
//use tinyboards_db_views::structs::SiteView;
//use tinyboards_federation::config::{FederationConfig, FederationMiddleware};
use tinyboards_routes::media;
use tinyboards_server::{
    api_routes, code_migrations::run_advanced_migrations, init_logging,
    root_span_builder::QuieterRootSpanBuilder, scheduled_tasks,
};
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};
use tracing_actix_web::TracingLogger;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// max timeout for http requests
pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[actix_web::main]
async fn main() -> Result<(), TinyBoardsError> {
    dotenv().ok(); // TODO - remove this (should be un-needed)

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

    // init the secret
    let db_url = get_db_url(Some(&settings));
    let secret = Secret::init(db_url).expect("Couldn't initialize secrets.");

    // make sure local site is setup
    /*let site_view = SiteView::read_local(&pool)
        .await
        .expect("local site is not set up");

    let local_site = site_view.local_site;
    let federation_enabled = local_site.federation_enabled;*/

    /*if federation_enabled {
        println!("federation is enabled, host is {}", &settings.hostname);
    }*/

    // make sure private instance and federation enabled are not turned on at the same time
    //check_private_instance_and_federation_enabled(&local_site)?;

    /*let rate_limit_config =
        local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
    let rate_limit_cell = RateLimitCell::new(rate_limit_config).await;*/

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
            //rate_limit_cell.clone(),
            graphql_schema.clone(),
        );

        let cors_config = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS", "PUT"])
            .allowed_headers(vec!["Content-Type", "Accepts"]);

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors_config)
            .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
            .app_data(Data::new(context))
            // the routes
            //.configure(|cfg| api_routes::config(cfg, &rate_limit_cell))
            // GraphQL
            .configure(api_routes::graphql_config)
            .configure(media::config)
    })
    .bind((settings_bind.bind, settings_bind.port))
    .map_err(|_| TinyBoardsError::from_message(500, "could not bind to ip"))?
    .run()
    .await
    .map_err(|_| TinyBoardsError::from_message(500, "could not start web server"))?;

    Ok(())
}

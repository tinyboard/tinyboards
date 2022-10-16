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
use doku::json::{AutoComments, Formatting};
use porpl_api_common::{utils::blocking, data::PorplContext};
use porpl_server::{
    api_routes,
    init_logging,
};
use porpl_utils::{
    error::PorplError,
    rate_limit::{rate_limiter::RateLimiter, RateLimit},
    settings::{structs::Settings, SETTINGS},
};
use porpl_db::utils::get_database_url_from_env;
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

    // let pool2 = pool.clone();
    // thread::spawn(move || {

    // })

    let rate_limiter = RateLimit {
        rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
        rate_limit_config: settings.rate_limit.to_owned().unwrap_or_default(),
    };

    // // init the secrets
    // let conn = &mut pool.get().unwrap();

    // let secret = Secret::init(conn).expect("Couldn't initialize secrets.");

    println!(
        "Starting http server at {}:{}",
        settings.bind, settings.port
    );

    let reqwest_client = Client::builder()
        .user_agent(build_user_agent(&settings))
        .timeout(REQWEST_TIMEOUT)
        .build()?;

    let settings_bind = settings.clone();
    HttpServer::new(move || {
        let context = PorplContext::create(

        );
        let rate_limiter = rate_limiter.clone();
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(TracingLogger::<QuieterRootSpanBuilder>::new())
            .app_data(Data::new(context))
            .app_data(Data::new(rate_limiter.clone()))
            .configure(|cfg| api_routes::config(cfg, &rate_limiter))
    })
    .bind((settings_bind.bind, settings_bind.port))?
    .run()
    .await?;

    Ok(())
}




































// use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};

// use porpl_api::Perform;
// use porpl_api_common::{
//     data::PorplContext,
//     user::{GetUser, Login, Register, Profile},
//     post::{SubmitPost, ListPosts, GetPost, CreatePostLike, DeletePost},
// };
// use porpl_api_crud::PerformCrud;

// use porpl_utils::PorplError;

// use dotenv::dotenv;
// use serde::Deserialize;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     HttpServer::new(|| {
//         App::new()
//             .app_data(web::Data::new(PorplContext::init()))
//             .service(
//                 web::scope("")
//                     .route("/@{username}", web::get().to(perform_get::<Profile>))   
//             )
//             .service(
//                 web::scope("/api/v1")
//                     .route("/signup", web::post().to(perform_post_crud::<Register>))
//                     .route("/login", web::get().to(perform_post::<Login>))
//                     .route(
//                         "/user/{username}",
//                         web::get().to(perform_get_crud::<GetUser>),
//                     ) 
//                     .route(
//                         "/post/submit",
//                         web::post().to(perform_post_crud::<SubmitPost>),
//                     )
//                     .route(
//                         "/post/list", 
//                         web::get().to(perform_get_crud::<ListPosts>),
//                     )
//                     .route(
//                         "/post/{post_id}",
//                         web::get().to(perform_get_crud::<GetPost>)
//                     )
//                     .route(
//                         "/post/vote",
//                         web::post().to(perform_post::<CreatePostLike>)
//                     )
//                     .route(
//                         "/post/delete",
//                         web::post().to(perform_post_crud::<DeletePost>)
//                     ),
//             )
//     })
//     .bind(("127.0.0.1", 8080))?
//     .run()
//     .await
// }

// async fn perform<'des, Request>(
//     data: Request,
//     context: web::Data<PorplContext>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: Perform<'des>,
//     Request: Send + 'static,
// {
//     let auth_header = req
//         .headers()
//         .get("Authorization")
//         .map(|header| header.to_str());
//     let auth_header = match auth_header {
//         Some(h) => match h {
//             Ok(h) => Some(h),
//             Err(_) => None,
//         },
//         None => None,
//     };

//     let res = data
//         .perform(&context, path.into_inner(), auth_header)
//         .await
//         .map(|json| HttpResponse::Ok().json(json))?;

//     Ok(res)
// }

// async fn perform_get<'des, Request>(
//     data: web::Data<PorplContext>,
//     query: web::Query<Request>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: Deserialize<'des> + Send + 'static + Perform<'des>,
// {
//     perform::<Request>(query.0, data, path, req).await
// }

// async fn perform_post<'des, Request>(
//     data: web::Data<PorplContext>,
//     body: web::Json<Request>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: Deserialize<'des> + Perform<'des> + Send + 'static,
// {
//     perform::<Request>(body.into_inner(), data, path, req).await
// }

// async fn perform_crud<'des, Request>(
//     data: Request,
//     context: web::Data<PorplContext>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: PerformCrud<'des>,
//     Request: Send + 'static,
// {
//     let auth_header = req
//         .headers()
//         .get("Authorization")
//         .map(|header| header.to_str());
//     let auth_header = match auth_header {
//         Some(h) => match h {
//             Ok(h) => Some(h),
//             Err(_) => None,
//         },
//         None => None,
//     };

//     let res = data
//         .perform(&context, path.into_inner(), auth_header)
//         .await
//         .map(|json| HttpResponse::Ok().json(json))?;

//     Ok(res)
// }

// async fn perform_get_crud<'des, Request>(
//     data: web::Data<PorplContext>,
//     query: web::Query<Request>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: Deserialize<'des> + Send + 'static + PerformCrud<'des>,
// {
//     perform_crud::<Request>(query.0, data, path, req).await
// }

// async fn perform_post_crud<'des, Request>(
//     data: web::Data<PorplContext>,
//     body: web::Json<Request>,
//     path: web::Path<Request::Route>,
//     req: HttpRequest,
// ) -> Result<HttpResponse, PorplError>
// where
//     Request: Deserialize<'des> + PerformCrud<'des> + Send + 'static,
// {
//     perform_crud::<Request>(body.into_inner(), data, path, req).await
// }

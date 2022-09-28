//mod data;
use actix_web::{web, App, HttpResponse, HttpServer, Result};

use porpl_api::data::PorplContext;
use porpl_api::users::CreateUser;
use porpl_api::users::GetUsers;
use porpl_api::Perform;
use porpl_utils::PorplError;

use serde::Deserialize;
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // get them environment variables at runtime yo!
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(PorplContext::init()))
            .route("/api/users", web::get().to(perform_get::<GetUsers>))
            .route("/api/signup", web::post().to(perform_post::<CreateUser>))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn perform<Request>(
    data: Request,
    context: web::Data<PorplContext>,
) -> Result<HttpResponse, PorplError>
where
    Request: Perform,
    Request: Send + 'static,
{
    let res = data
        .perform(&context)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;

    Ok(res)
}

async fn perform_get<'a, T>(
    data: web::Data<PorplContext>,
    query: web::Query<T>,
) -> Result<HttpResponse, PorplError>
where
    T: Deserialize<'a> + Send + 'static + Perform,
{
    perform::<T>(query.0, data).await
}

async fn perform_post<'a, T>(
    data: web::Data<PorplContext>,
    body: web::Json<T>,
) -> Result<HttpResponse, PorplError>
where
    T: Deserialize<'a> + Perform + Send + 'static,
{
    perform::<T>(body.into_inner(), data).await
}

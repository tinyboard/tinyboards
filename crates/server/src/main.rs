//mod data;
use actix_web::{web, App, HttpResponse, HttpServer, Result};

use porpl_api::data::PorplContext;
use porpl_api::error::PorplError;
use porpl_api::post::GetPosts;
use porpl_api::users::GetUsers;
use porpl_api::Perform;

use serde::Deserialize;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(PorplContext::init()))
            .route("/", web::get().to(perform_get::<GetPosts>))
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

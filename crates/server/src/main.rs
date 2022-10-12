//mod data;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};

use porpl_api::Perform;
use porpl_api_common::{
    data::PorplContext,
    person::{GetUser, Login, Register},
    post::{SubmitPost, ListPosts},
};
use porpl_api_crud::PerformCrud;

use porpl_utils::PorplError;

use dotenv::dotenv;
use serde::Deserialize;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(PorplContext::init()))
            .service(
                web::scope("/api/v1")
                    .route("/signup", web::post().to(perform_post_crud::<Register>))
                    .route("/login", web::get().to(perform_post::<Login>))
                    .route(
                        "/user/{username}",
                        web::get().to(perform_get_crud::<GetUser>),
                    ) // example api endpoint for testing extractor (I think)
                    .route(
                        "/post/submit",
                        web::post().to(perform_post_crud::<SubmitPost>),
                    )
                    .route(
                        "/post/list", 
                        web::get().to(perform_get_crud::<ListPosts>),
                    ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn perform<'des, Request>(
    data: Request,
    context: web::Data<PorplContext>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Perform<'des>,
    Request: Send + 'static,
{
    let auth_header = req
        .headers()
        .get("Authorization")
        .map(|header| header.to_str());
    let auth_header = match auth_header {
        Some(h) => match h {
            Ok(h) => Some(h),
            Err(_) => None,
        },
        None => None,
    };

    let res = data
        .perform(&context, path.into_inner(), auth_header)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;

    Ok(res)
}

async fn perform_get<'des, Request>(
    data: web::Data<PorplContext>,
    query: web::Query<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Deserialize<'des> + Send + 'static + Perform<'des>,
{
    perform::<Request>(query.0, data, path, req).await
}

async fn perform_post<'des, Request>(
    data: web::Data<PorplContext>,
    body: web::Json<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Deserialize<'des> + Perform<'des> + Send + 'static,
{
    perform::<Request>(body.into_inner(), data, path, req).await
}

async fn perform_crud<'des, Request>(
    data: Request,
    context: web::Data<PorplContext>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: PerformCrud<'des>,
    Request: Send + 'static,
{
    let auth_header = req
        .headers()
        .get("Authorization")
        .map(|header| header.to_str());
    let auth_header = match auth_header {
        Some(h) => match h {
            Ok(h) => Some(h),
            Err(_) => None,
        },
        None => None,
    };

    let res = data
        .perform(&context, path.into_inner(), auth_header)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;

    Ok(res)
}

async fn perform_get_crud<'des, Request>(
    data: web::Data<PorplContext>,
    query: web::Query<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Deserialize<'des> + Send + 'static + PerformCrud<'des>,
{
    perform_crud::<Request>(query.0, data, path, req).await
}

async fn perform_post_crud<'des, Request>(
    data: web::Data<PorplContext>,
    body: web::Json<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Deserialize<'des> + PerformCrud<'des> + Send + 'static,
{
    perform_crud::<Request>(body.into_inner(), data, path, req).await
}

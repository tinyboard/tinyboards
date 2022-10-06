//mod data;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};

use porpl_api::{
    data::{PorplContext},
    Perform,
};

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
                    // .route("/users", web::get().to(perform_get::<GetUsers>))
                    // .route("/signup", web::post().to(perform_post::<CreateUser>))
                    // .route("/login", web::post().to(perform_post::<UserLogin>))
                    // .route("/me", web::get().to(perform_get::<GetLoggedInUser>))
                    // .route("/posts/submit", web::post().to(perform_post::<CreateSubmission>))
                    // .route("/comments/submit", web::post().to(perform_post::<CreateComment>))
                )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn perform<Request>(
    data: Request,
    context: web::Data<PorplContext>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    Request: Perform,
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
        .perform(&context, auth_header)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;

    Ok(res)
}

async fn perform_get<'a, T>(
    data: web::Data<PorplContext>,
    query: web::Query<T>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    T: Deserialize<'a> + Send + 'static + Perform,
{
    perform::<T>(query.0, data, req).await
}

async fn perform_post<'a, T>(
    data: web::Data<PorplContext>,
    body: web::Json<T>,
    req: HttpRequest,
) -> Result<HttpResponse, PorplError>
where
    T: Deserialize<'a> + Perform + Send + 'static,
{
    perform::<T>(body.into_inner(), data, req).await
}

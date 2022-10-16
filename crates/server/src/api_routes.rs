use actix_web::*;
use porpl_api::{Perform, data::PorplContext};
use porpl_api_common::{
    person::*,
    post::*,
};
use porpl_api_crud::PerformCrud;
use porpl_utils::rate_limit::RateLimit;
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
    cfg.service(
        web::scope("/api/v1")
        // User
        .service(
            web::resource("/signup")
                .guard(guard::Post())
                .wrap(rate_limit.register())
                .route(web::post().to(route_post_crud::<Register>))
        )
        .service(
            web::scope("/user")
                .wrap(rate_limit.message())
                .route("/login", web::post().to(route_post::<Login>))
                .route("/{username}", web::get().to(route_get_crud::<GetUser>))
                .route("/{username}", web::put().to(route_get_crud::<UpdateUser>))
                .route("/{username}", web::delete().to(route_get_crud::<DeleteUser>))
        )
        // Post
        .service(
            web::scope("/post")
              .wrap(rate_limit.message())
              .route("/{post_id}", web::get().to(route_get_crud::<GetPost>))
              .route("/{post_id}", web::post().to(route_get_crud::<SubmitPost>))
              .route("/{post_id}", web::put().to(route_get_crud::<UpdatePost>))
              .route("/{post_id}", web::delete().to(route_get_crud::<DeletePost>))
              .route("/list", web::get().to(route_get_crud::<ListPosts>))
              .route("/sticky", web::put().to(route_get_crud::<StickyPost>))
              .route("/report", web::put().to(route_get_crud::<ReportPost>))
              .route("/vote", web::post().to(route_post::<CreatePostLike>))
              .route("/vote", web::put().to(route_post::<UpdatePostLike>))
              .route("/vote", web::delete().to(route_post::<DeletePostLike>))
        )
    )
}


async fn perform<Request>(
    data: Request,
    context: web::Data<PorplContext>,
) -> Result<HttpResponse, Error> 
where
    Request: Perform,
    Request: Send + 'static,
{
    let res = data
        .perform(&context, None)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;
    Ok(res)
}

async fn route_get<'a, Data>(
    data: web::Query<Data>,
    context: web::Data<PorplContext>,
) -> Result<HttpResponse, Error>
where
    Data: Deserialize<'a> + Send + 'static + Perform,
{
    perform::<Data>(data.0, context).await
}

async fn route_post<'a, Data>(
    data: web::Json<Data>,
    context: web::Data<PorplContext>,
  ) -> Result<HttpResponse, Error>
  where
    Data: Deserialize<'a> + Send + 'static + Perform,
  {
    perform::<Data>(data.0, context).await
  }
  
  async fn perform_crud<Request>(
    data: Request,
    context: web::Data<PorplContext>,
  ) -> Result<HttpResponse, Error>
  where
    Request: PerformCrud,
    Request: Send + 'static,
  {
    let res = data
      .perform(&context, None)
      .await
      .map(|json| HttpResponse::Ok().json(json))?;
    Ok(res)
  }
  
  async fn route_get_crud<'a, Data>(
    data: web::Query<Data>,
    context: web::Data<PorplContext>,
  ) -> Result<HttpResponse, Error>
  where
    Data: Deserialize<'a> + Send + 'static + PerformCrud,
  {
    perform_crud::<Data>(data.0, context).await
  }
  
  async fn route_post_crud<'a, Data>(
    data: web::Json<Data>,
    context: web::Data<PorplContext>,
  ) -> Result<HttpResponse, Error>
  where
    Data: Deserialize<'a> + Send + 'static + PerformCrud,
  {
    perform_crud::<Data>(data.0, context).await
  }
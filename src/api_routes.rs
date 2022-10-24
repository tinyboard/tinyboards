use actix_web::*;
use porpl_api::Perform;
use porpl_api_common::{
    comment::{CreateComment, GetPostComments, ListComments, CreateCommentLike, SaveComment, DeleteComment},
    data::PorplContext,
    post::*,
    user::*,
};
use porpl_api_crud::PerformCrud;
use porpl_utils::{rate_limit::RateLimit, PorplError};
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::resource("/signup")
                    .guard(guard::Post())
                    .wrap(rate_limit.register())
                    .route(web::post().to(route_post_crud::<Register>)),
            )
            .service(
                web::resource("/login")
                    .guard(guard::Post())
                    .wrap(rate_limit.message())
                    .route(web::post().to(route_post::<Login>)),
            )
            .service(
                web::scope("/user")
                    .route("/@{username}", web::get().to(route_get::<Profile>))
                    .route("/me", web::get().to(route_get::<GetLoggedInUser>)),
            )
            // Post
            .service(
                web::scope("/post")
                    .wrap(rate_limit.message())
                    .service(
                        web::resource("")
                            .route(web::get().to(route_get_crud::<ListPosts>))    
                            .guard(guard::Get())
                            .wrap(rate_limit.post())
                    )
                    //.route("", web::get().to(route_get_crud::<ListPosts>))
                    .route("/{post_id}", web::get().to(route_get_crud::<GetPost>))
                    .route("/{post_id}/comments",web::get().to(route_get_crud::<GetPostComments>))
                    .route("/submit", web::post().to(route_post_crud::<SubmitPost>))
                    .route("/vote", web::post().to(route_post::<CreatePostLike>))
                    .route("/save", web::post().to(route_post::<SavePost>))
                    .route("/delete", web::post().to(route_post_crud::<DeletePost>)),
            )
            // Comments
            .service(
                web::scope("/comments")
                    .wrap(rate_limit.message())
                    .route("", web::get().to(route_get_crud::<ListComments>))
                    .route("/submit", web::post().to(route_post_crud::<CreateComment>))
                    .route("/vote", web::post().to(route_post::<CreateCommentLike>))
                    .route("/save", web::post().to(route_post::<SaveComment>))
                    .route("/delete", web::post().to(route_post_crud::<DeleteComment>)),
            ),
    );
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

async fn route_get<'des, Request>(
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

async fn route_post<'des, Request>(
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

async fn route_get_crud<'des, Request>(
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

async fn route_post_crud<'des, Request>(
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

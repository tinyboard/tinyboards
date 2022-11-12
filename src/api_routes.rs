use actix_web::*;
use serde::Deserialize;
use tinyboards_api::{Perform, PerformQuery};
use tinyboards_api_common::{
    comment::*, 
    moderator::*, 
    site::*,
    post::*,
    user::*,
    data::TinyBoardsContext,  
};
use tinyboards_api_crud::PerformCrud;
use tinyboards_utils::{rate_limit::RateLimit, TinyBoardsError};

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
    cfg.service(
        web::scope("")
            .route("/feed", web::get().to(route_get_query::<GetFeed>)),
    ).service(
        web::scope("/api/v1")
            .route("/me", web::get().to(route_get::<GetLoggedInUser>))
            // Authenticate
            .service(
                web::scope("/auth")
                    //.guard(guard::Post())
                    //.wrap(rate_limit.message())
                    .route("/login", web::post().to(route_post::<Login>))
                    .route("/signup", web::post().to(route_post_crud::<Register>)),
            )
            // User
            .service(
                web::scope("/user").route("/{username}", web::get().to(route_get::<Profile>)), //.route("/me", web::get().to(route_get::<GetLoggedInUser>))
                                                                                               //.guard(guard::Post())
                                                                                               //.wrap(rate_limit.register())
            )
            // Post
            .service(
                web::scope("/posts")
                    //.wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<SubmitPost>))
                    //.guard(guard::Post())
                    .route("", web::get().to(route_get_crud::<ListPosts>))
                    //.guard(guard::Get())
                    .route("/{post_id}", web::get().to(route_get_crud::<GetPost>))
                    //.guard(guard::Get())
                    .route(
                        "/{post_id}",
                        web::delete().to(route_post_crud::<DeletePost>),
                    )
                    //.guard(guard::Delete())
                    .route("/{post_id}", web::put().to(route_post_crud::<EditPost>))
                    //.guard(guard::Put())
                    .route(
                        "/{post_id}/vote",
                        web::post().to(route_post::<CreatePostVote>),
                    )
                    //.guard(guard::Post())
                    .route("/{post_id}/save", web::post().to(route_post::<SavePost>))
                    //.guard(guard::Post())
                    .route(
                        "/{post_id}/comments",
                        web::get().to(route_get_crud::<GetPostComments>),
                    ),
            )
            // Comment
            .service(
                web::scope("/comments").route("", web::get().to(route_get_crud::<ListComments>)), //.guard(guard::Get())
            )
            .service(
                web::scope("/comment")
                    //.wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<CreateComment>))
                    //.guard(guard::Post())
                    .route("/{comment_id}", web::get().to(route_get_crud::<GetComment>))
                    //.guard(guard::Get())
                    .route(
                        "/{comment_id}",
                        web::delete().to(route_post_crud::<DeleteComment>),
                    )
                    //.guard(guard::Delete())
                    .route(
                        "/{comment_id}",
                        web::put().to(route_post_crud::<EditComment>),
                    )
                    //.guard(guard::Put())
                    .route(
                        "/{comment_id}/vote",
                        web::post().to(route_post::<CreateCommentVote>),
                    )
                    //.guard(guard::Post())
                    .route(
                        "/{comment_id}/save",
                        web::post().to(route_post::<SaveComment>),
                    ), //.guard(guard::Post()),
            )
            // Mod & Admin Actions
            .service(
                web::scope("/mod").route("/lock_post", web::post().to(route_post::<LockPost>)), //.route("/ban", web::post().to(route_post::<BanUser>)),
            ),
    );
}

async fn perform<'des, Request>(
    data: Request,
    context: web::Data<TinyBoardsContext>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
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

async fn perform_query<'des, Request>(
    data: Request,
    context: web::Data<TinyBoardsContext>,
    params: web::Query<Request::QueryForm>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: PerformQuery<'des>,
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
        .perform_query(&context, actix_web::web::Query(params.into_inner()), auth_header)
        .await
        .map(|json| HttpResponse::Ok().json(json))?;

    Ok(res)
}

async fn route_get<'des, Request>(
    data: web::Data<TinyBoardsContext>,
    query: web::Query<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: Deserialize<'des> + Send + 'static + Perform<'des>,
{
    perform::<Request>(query.0, data, path, req).await
}


async fn route_get_query<'des, Request>(
    data: web::Data<TinyBoardsContext>,
    query: web::Query<Request>,
    params: web::Query<Request::QueryForm>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: Deserialize<'des> + Send + 'static + PerformQuery<'des>,
{
    perform_query::<Request>(query.0, data, params, req).await
}

async fn route_post<'des, Request>(
    data: web::Data<TinyBoardsContext>,
    body: web::Json<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: Deserialize<'des> + Perform<'des> + Send + 'static,
{
    perform::<Request>(body.into_inner(), data, path, req).await
}

async fn perform_crud<'des, Request>(
    data: Request,
    context: web::Data<TinyBoardsContext>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
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
    data: web::Data<TinyBoardsContext>,
    query: web::Query<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: Deserialize<'des> + Send + 'static + PerformCrud<'des>,
{
    perform_crud::<Request>(query.0, data, path, req).await
}

async fn route_post_crud<'des, Request>(
    data: web::Data<TinyBoardsContext>,
    body: web::Json<Request>,
    path: web::Path<Request::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Request: Deserialize<'des> + PerformCrud<'des> + Send + 'static,
{
    perform_crud::<Request>(body.into_inner(), data, path, req).await
}

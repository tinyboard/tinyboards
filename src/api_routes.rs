use actix_multipart::Multipart;
use actix_web::*;
use serde::Deserialize;
use tinyboards_api::{Perform, PerformUpload};
use tinyboards_api_common::{
    admin::*, applications::*, board::*, comment::*, data::TinyBoardsContext, moderator::*,
    person::*, post::*, site::*,
};
use tinyboards_api_crud::PerformCrud;
use tinyboards_apub::{SendActivity, api::PerformApub};
//use tinyboards_federation::config::Data;
use tinyboards_utils::{rate_limit::RateLimitCell, TinyBoardsError};

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
    cfg.service(
        web::scope("/api/v1")
            .route("/me", web::get().to(route_get::<GetLoggedInUser>))
            .route("/feed", web::get().to(route_get::<GetFeed>))
            .route("/members", web::get().to(route_get::<GetMembers>))
            .route("/search", web::get().to(route_get_apub::<Search>))
            .route("/settings", web::get().to(route_get::<GetUserSettings>))
            .route("/settings", web::put().to(route_post::<SaveUserSettings>))
            //.route("/remove", web::post().to(route_post::<RemoveObject>))
            //.route("/approve", web::post().to(route_post::<ApproveObject>))
            .route(
                "/password_reset",
                web::post().to(route_post::<PasswordResetRequest>),
            )
            .route(
                "/password_reset/{reset_token}",
                web::post().to(route_post::<ExecutePasswordReset>),
            )
            // Get Federated Instances
            .service(
                web::scope("/federated_instances")
                    .wrap(rate_limit.message())
                    .route("", web::get().to(route_get::<GetFederatedInstances>)),
            )
            // Validate Site Invite
            .route(
                "/validate_invite/{invite_token}",
                web::post().to(route_post::<ValidateSiteInvite>),
            )
            // File Upload / Deletion
            .service(
                web::scope("/file")
                    .route("/upload", web::put().to(upload_file::<Multipart>))
                    .route("/{file_name}", web::delete().to(route_post::<DeleteFile>)),
            )
            // File Retrieval
            //.route("/media/{file_name}", web::get().to(route_get::<GetFile>))
            // Authenticate
            .service(
                web::scope("/auth")
                    //.wrap(rate_limit.message())
                    .route("/login", web::post().to(route_post::<Login>))
                    .route("/signup", web::post().to(route_post_crud::<Register>))
                    // Delete Account
                    .route(
                        "/delete_account",
                        web::post().to(route_post_crud::<DeleteAccount>),
                    ),
            )
            // User
            .service(web::scope("/names").route("", web::get().to(route_get::<SearchNames>)))
            .service(
                web::scope("/user")
                    .route("/{username}", web::get().to(route_get::<Profile>))
                    .route(
                        "/verify_email/{token}",
                        web::post().to(route_post::<VerifyEmail>),
                    ),
            )
            // Notifications
            .service(
                web::scope("/notifications")
                    .wrap(rate_limit.message())
                    .route("/unread", web::get().to(route_get::<GetUnreadCount>))
                    .route("/mentions", web::get().to(route_get::<GetPersonMentions>))
                    .route(
                        "/mentions/mark_read",
                        web::post().to(route_post::<MarkAllMentionsRead>),
                    )
                    .route("/replies", web::get().to(route_get::<GetCommentReplies>))
                    .route(
                        "/replies/mark_read",
                        web::post().to(route_post::<MarkAllRepliesRead>),
                    ),
            )
            // Board
            .service(
                web::scope("/boards")
                    .wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<CreateBoard>))
                    .route("/remove", web::post().to(route_post_crud::<RemoveBoard>))
                    .route("/subscribe", web::post().to(route_post::<SubscribeToBoard>))
                    .route("/block", web::post().to(route_post::<BlockBoard>))
                    .route("/{board_id}", web::put().to(route_post_crud::<EditBoard>))
                    .route(
                        "/{board_id}",
                        web::delete().to(route_post_crud::<DeleteBoard>),
                    ),
            )
            // Post
            .service(
                web::scope("/posts")
                    .wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<SubmitPost>))
                    .route("", web::get().to(route_get_crud::<ListPosts>))
                    .route("/remove", web::post().to(route_post_crud::<RemovePost>))
                    .route("/lock", web::post().to(route_post::<LockPost>))
                    .route("/report", web::post().to(route_post::<CreatePostReport>))
                    .route("/report/list", web::post().to(route_post::<ListPostReports>))
                    .route("/report/resolve", web::post().to(route_post::<ResolvePostReport>))
                    .route("/{post_id}", web::get().to(route_get_crud::<GetPost>))
                    .route(
                        "/{post_id}",
                        web::delete().to(route_post_crud::<DeletePost>),
                    )
                    .route("/{post_id}", web::put().to(route_post_crud::<EditPost>))
                    .route(
                        "/{post_id}/vote",
                        web::post().to(route_post::<CreatePostVote>),
                    )
                    .route("/{post_id}/save", web::post().to(route_post::<SavePost>))
                    .route(
                        "/{post_id}/comments",
                        web::get().to(route_get_crud::<GetPostComments>),
                    ),
            )
            // Comment
            .service(
                web::scope("/comments")
                    .wrap(rate_limit.message())
                    .route("", web::get().to(route_get_crud::<ListComments>))
                    .route("", web::post().to(route_post_crud::<CreateComment>))
                    .route("/remove", web::post().to(route_post_crud::<RemoveComment>))
                    .route("/report", web::post().to(route_post::<CreateCommentReport>))
                    .route("/report/list", web::post().to(route_post::<ListCommentReports>))
                    .route("/report/resolve", web::post().to(route_post::<ResolveCommentReport>))
                    .route("/{comment_id}", web::get().to(route_get_crud::<GetComment>))
                    .route(
                        "/{comment_id}",
                        web::delete().to(route_post_crud::<DeleteComment>),
                    )
                    .route(
                        "/{comment_id}",
                        web::put().to(route_post_crud::<EditComment>),
                    )
                    .route(
                        "/{comment_id}/vote",
                        web::post().to(route_post::<CreateCommentVote>),
                    )
                    .route(
                        "/{comment_id}/save",
                        web::post().to(route_post::<SaveComment>),
                    ),
            )
            // Mod Actions
            .service(
                web::scope("/mod")
                    .route("/ban", web::post().to(route_post::<BanUser>))
                    .route("/board_ban", web::post().to(route_post::<BanFromBoard>))
                    .route("/ban_board", web::post().to(route_post::<BanBoard>))
                    .route("/feature_post", web::post().to(route_post::<FeaturePost>))
                    .route("/add_moderator", web::post().to(route_post::<AddBoardMod>)),
            )
            // Admin Actions
            .service(
                web::scope("/admin")
                    .route("/add_admin", web::post().to(route_post::<AddAdmin>))
                    .route("/leave_admin", web::post().to(route_post::<LeaveAdmin>))
                    .route("/purge_user", web::post().to(route_post::<PurgePerson>))
                    .route("/purge_post", web::post().to(route_post::<PurgePost>))
                    .route("/purge_comment", web::post().to(route_post::<PurgeComment>))
                    .route("/purge_board", web::post().to(route_post::<PurgeBoard>))
                    .route("/hide_board", web::post().to(route_post::<HideBoard>))
                    .route(
                        "/site_settings",
                        web::get().to(route_get::<GetSiteSettings>),
                    )
                    .route(
                        "/site_settings",
                        web::put().to(route_post::<SaveSiteSettings>),
                    )
                    .route(
                        "/invite",
                        web::post().to(route_post_crud::<CreateSiteInvite>),
                    )
                    .route("/invite", web::get().to(route_get_crud::<ListSiteInvites>))
                    .route(
                        "/invite/{invite_id}",
                        web::delete().to(route_post_crud::<DeleteSiteInvite>),
                    )
                    .route(
                        "/application",
                        web::get().to(route_get_crud::<ListRegistrationApplications>),
                    )
                    .route(
                        "/application/{app_id}",
                        web::post().to(route_post::<HandleRegistrationApplication>),
                    ),
            ),
    );
}

fn get_auth(req: &HttpRequest) -> Option<&str> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .map(|header| header.to_str());

    match auth_header {
        Some(h) => match h {
            Ok(h) => Some(h),
            Err(_) => None,
        },
        None => None,
    }
}

async fn perform<'a, Data>(
    data: Data,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<Data::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Data: Perform<'a>
        + SendActivity<Response = <Data as Perform<'a>>::Response>
        + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    let auth_header = get_auth(&req);
    let res = data.clone().perform(&context, path.into_inner(), auth_header).await?;
    SendActivity::send_activity(&data, &res, &apub_data, auth_header).await?;
    Ok(HttpResponse::Ok().json(res))
}

async fn route_get<'a, Data>(
    data: web::Query<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<Data::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError> 
where
    Data: Perform<'a>
        + SendActivity<Response = <Data as Perform<'a>>::Response>
        + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    perform::<Data>(data.0, context, apub_data, path, req).await
}

async fn route_get_apub<'a, Data>(
    req: HttpRequest,
    data: web::Query<Data>,
    context: tinyboards_federation::config::Data<TinyBoardsContext>,
) -> Result<HttpResponse, Error>
where
    Data: PerformApub
        + SendActivity<Response = <Data as PerformApub>::Response>
        + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{   
    let auth_header = get_auth(&req);
    let res = data.perform(&context, auth_header).await?;
    SendActivity::send_activity(&data.0, &res, &context, auth_header).await?;
    Ok(HttpResponse::Ok().json(res))
}


async fn route_post<'a, Data>(
    data: web::Json<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<Data::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Data: Perform<'a>
        + SendActivity<Response = <Data as Perform<'a>>::Response>
        + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    perform::<Data>(data.0, context, apub_data, path, req).await
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

async fn upload_file<'des, Multipart>(
    data: web::Data<TinyBoardsContext>,
    payload: Multipart,
    path: web::Path<Multipart::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Multipart: PerformUpload<'des> + 'static,
{
    let result = Multipart::perform_upload(
        payload,
        &data,
        path.into_inner(),
        req.headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok()),
    )
    .await?;
    Ok(HttpResponse::Ok().json(result))
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

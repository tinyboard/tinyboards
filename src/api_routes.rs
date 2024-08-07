use actix_multipart::Multipart;
use actix_web::*;
use serde::Deserialize;
use tinyboards_api::{Perform, PerformUpload};
use tinyboards_api_common::{
    admin::*, applications::*, board::*, comment::*, data::TinyBoardsContext, emoji::*,
    message::GetMessages, moderator::*, person::*, post::*, site::*,
};
use tinyboards_api_crud::PerformCrud;
use tinyboards_apub::{api::PerformApub, SendActivity};
use tinyboards_utils::{rate_limit::RateLimitCell, TinyBoardsError};

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
    cfg.service(
        web::scope("/api/v1")
            .route("/me", web::get().to(route_get::<GetLoggedInUser>))
            .route("/members", web::get().to(route_get::<GetMembers>))
            .route("/search", web::get().to(route_get_apub::<Search>))
            .route("/settings", web::get().to(route_get::<GetUserSettings>))
            .route("/settings", web::put().to(route_post::<SaveUserSettings>))
            .route("/messages", web::get().to(route_get_crud::<GetMessages>))
            // resolve federated objects (object => post, person, board or comment)
            .route(
                "/resolve_object",
                web::get().to(route_get_apub::<ResolveObject>),
            )
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
                    .route("", web::get().to(route_get_apub::<GetPersonDetails>))
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
                    )
                    .route(
                        "/messages/mark_read",
                        web::post().to(route_post::<MarkAllMessagesRead>),
                    ),
            )
            // Subscriptions
            .service(
                web::scope("/subscriptions")
                    .wrap(rate_limit.message())
                    .route(
                        "/boards",
                        web::post().to(route_post_crud::<SubscribeToBoard>),
                    )
                    .route(
                        "/boards/{board_id}",
                        web::delete().to(route_post_crud::<UnsubFromBoard>),
                    ),
            )
            // Board
            .service(
                web::scope("/boards")
                    .wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<CreateBoard>))
                    .route("/get", web::get().to(route_get_apub::<GetBoard>))
                    .route("/exists", web::get().to(route_get_crud::<CheckBoardExists>))
                    .route(
                        "/{board_id}/banned",
                        web::patch().to(route_post_crud::<ToggleBoardBan>),
                    )
                    //.route("/subscribe", web::post().to(route_post::<SubscribeToBoard>))
                    .route("/block", web::post().to(route_post::<BlockBoard>))
                    .route("/{board_id}", web::put().to(route_post_crud::<EditBoard>))
                    .route(
                        "/{board_id}",
                        web::delete().to(route_post_crud::<DeleteBoard>),
                    )
                    .route(
                        "/{board_id}/mods",
                        web::get().to(route_get_crud::<ListBoardMods>),
                    )
                    .route(
                        "/{board_id}/mods",
                        web::put().to(route_post_crud::<InviteBoardMod>),
                    )
                    .route(
                        "/{board_id}/mods",
                        web::post().to(route_post_crud::<AddBoardMod>),
                    )
                    .route(
                        "/{board_id}/mods/{person_id}",
                        web::patch().to(route_post_crud::<EditBoardMod>),
                    )
                    .route(
                        "/{board_id}/mods/{person_id}",
                        web::delete().to(route_post_crud::<RemoveBoardMod>),
                    ),
            )
            // Post
            .service(
                web::scope("/posts")
                    .wrap(rate_limit.message())
                    .route("", web::post().to(route_post_crud::<SubmitPost>))
                    .route("", web::get().to(route_get_apub::<GetPosts>))
                    .route(
                        "/{post_id}/removed",
                        web::patch().to(route_post_crud::<TogglePostRemove>),
                    )
                    .route(
                        "/{post_id}/locked",
                        web::patch().to(route_post_crud::<TogglePostLock>),
                    )
                    .route(
                        "/{post_id}/reports",
                        web::post().to(route_post_crud::<CreatePostReport>),
                    )
                    .route(
                        "/{post_id}/reports",
                        web::get().to(route_get_crud::<GetPostReports>),
                    )
                    .route(
                        "/{post_id}/featured",
                        web::patch().to(route_post_crud::<TogglePostFeatured>),
                    )
                    /*.route(
                        "/report/list",
                        web::post().to(route_post::<ListPostReports>),
                    )
                    .route(
                        "/report/resolve",
                        web::post().to(route_post::<ResolvePostReport>),
                    )*/
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
                    .route("/{post_id}/saved", web::patch().to(route_post::<SavePost>))
                    .route(
                        "/{post_id}/comments",
                        web::get().to(route_get_crud::<GetPostComments>),
                    )
                    .route(
                        "/{post_id}/comments",
                        web::post().to(route_post_crud::<CreateComment>),
                    ),
            )
            // Comment
            .service(
                web::scope("/comments")
                    .wrap(rate_limit.message())
                    .route(
                        "/{comment_id}/removed",
                        web::patch().to(route_post_crud::<ToggleCommentRemove>),
                    )
                    .route("", web::get().to(route_get_apub::<GetComments>))
                    .route(
                        "/{comment_id}/reports",
                        web::post().to(route_post_crud::<CreateCommentReport>),
                    )
                    .route(
                        "/{comment_id}/reports",
                        web::get().to(route_get_crud::<GetCommentReports>),
                    )
                    /*.route(
                        "/report/list",
                        web::post().to(route_post::<ListCommentReports>),
                    )
                    .route(
                        "/report/resolve",
                        web::post().to(route_post::<ResolveCommentReport>),
                    )*/
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
                        "/{comment_id}/saved",
                        web::patch().to(route_post::<SaveComment>),
                    ),
            )
            // Mod Actions
            .service(
                web::scope("/mod")
                    .route("/board_ban", web::post().to(route_post::<BanFromBoard>))
                    //.route("/ban_board", web::post().to(route_post::<BanBoard>))
                    //.route("/feature_post", web::post().to(route_post::<TogglePostFeatured>))
                    //.route("", web::post().to(route_post::<AddBoardMod>))
                    .route("/queue/posts", web::get().to(route_get::<PostModQueue>))
                    .route(
                        "/queue/comments",
                        web::get().to(route_get::<CommentModQueue>),
                    ),
            )
            // Admin Actions
            .service(
                web::scope("/admin")
                    .route("/ban", web::post().to(route_post::<ToggleBan>))
                    .route("/add_admin", web::post().to(route_post::<AddAdmin>))
                    .route("/leave_admin", web::post().to(route_post::<LeaveAdmin>))
                    .route("/purge_user", web::post().to(route_post::<PurgePerson>))
                    .route("/purge_post", web::post().to(route_post::<PurgePost>))
                    .route("/purge_comment", web::post().to(route_post::<PurgeComment>))
                    .route("/purge_board", web::post().to(route_post::<PurgeBoard>))
                    .route("/hide_board", web::post().to(route_post::<HideBoard>))
                    .route(
                        "/banned_users",
                        web::get().to(route_get::<ListBannedPersons>),
                    )
                    .route("/site", web::get().to(route_get::<GetSiteSettings>))
                    .route("/site", web::put().to(route_post::<SaveSiteSettings>))
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
                    )
                    // Custom Emojis
                    .route("/emoji", web::post().to(route_post_crud::<CreateEmoji>))
                    .route("/emoji", web::put().to(route_post_crud::<EditEmoji>))
                    .route(
                        "/emoji/{emoji_id}",
                        web::delete().to(route_post_crud::<DeleteEmoji>),
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
    path: web::Path<<Data as Perform<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Data: Perform<'a>
        + SendActivity<
            Response = <Data as Perform<'a>>::Response,
            Route = <Data as Perform<'a>>::Route,
        > + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    let auth_header = get_auth(&req);

    let path = path.into_inner();
    let path_clone = path.clone();

    let res = data.clone().perform(&context, path, auth_header).await?;
    SendActivity::send_activity(&data, &res, &apub_data, &path_clone, auth_header).await?;
    Ok(HttpResponse::Ok().json(res))
}

async fn route_get<'a, Data>(
    data: web::Query<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<<Data as Perform<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Data: Perform<'a>
        + SendActivity<
            Response = <Data as Perform<'a>>::Response,
            Route = <Data as Perform<'a>>::Route,
        > + Clone
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
    path: web::Path<()>,
) -> Result<HttpResponse, Error>
where
    Data: PerformApub
        + SendActivity<Response = <Data as PerformApub>::Response, Route = ()>
        + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    let auth_header = get_auth(&req);
    let res = data.perform(&context, auth_header).await?;
    SendActivity::send_activity(&data.0, &res, &context, &path, auth_header).await?;
    Ok(HttpResponse::Ok().json(res))
}

async fn route_post<'a, Data>(
    data: web::Json<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<<Data as Perform<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError>
where
    Data: Perform<'a>
        + SendActivity<
            Response = <Data as Perform<'a>>::Response,
            Route = <Data as Perform<'a>>::Route,
        > + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    perform::<Data>(data.0, context, apub_data, path, req).await
}

async fn perform_crud<'a, Data>(
    data: Data,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<<Data as PerformCrud<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, Error>
where
    Data: PerformCrud<'a>
        + SendActivity<
            Response = <Data as PerformCrud<'a>>::Response,
            Route = <Data as PerformCrud<'a>>::Route,
        > + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    let auth_header = get_auth(&req);

    let path = path.into_inner();
    let path_clone = path.clone();

    let res = data.clone().perform(&context, path, auth_header).await?;
    SendActivity::send_activity(&data, &res, &apub_data, &path_clone, auth_header).await?;
    Ok(HttpResponse::Ok().json(res))
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

async fn route_post_crud<'a, Data>(
    data: web::Json<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<<Data as PerformCrud<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, Error>
where
    Data: PerformCrud<'a>
        + SendActivity<
            Response = <Data as PerformCrud<'a>>::Response,
            Route = <Data as PerformCrud<'a>>::Route,
        > + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    perform_crud::<Data>(data.0, context, apub_data, path, req).await
}

async fn route_get_crud<'a, Data>(
    data: web::Query<Data>,
    context: web::Data<TinyBoardsContext>,
    apub_data: tinyboards_federation::config::Data<TinyBoardsContext>,
    path: web::Path<<Data as PerformCrud<'a>>::Route>,
    req: HttpRequest,
) -> Result<HttpResponse, Error>
where
    Data: PerformCrud<'a>
        + SendActivity<
            Response = <Data as PerformCrud<'a>>::Response,
            Route = <Data as PerformCrud<'a>>::Route,
        > + Clone
        + Deserialize<'a>
        + Send
        + 'static,
{
    perform_crud::<Data>(data.0, context, apub_data, path, req).await
}

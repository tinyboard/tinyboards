use actix_web::*;
use async_graphql::dataloader::DataLoader;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
//use tinyboards_api::{Perform, PerformUpload};
use tinyboards_api::{context::TinyBoardsContext, utils::auth::get_user_from_header_opt};
use tinyboards_api::{LoggedInUser, MasterKey, PostgresLoader, Settings as GQLSettings};
//use tinyboards_apub::{api::PerformApub, SendActivity};
//use tinyboards_utils::{rate_limit::RateLimitCell, TinyBoardsError};

/*pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimitCell) {
    cfg.service(
        web::scope("/api/v1")
            /*.route("/me", web::get().to(route_get::<GetLoggedInUser>))
            .route("/members", web::get().to(route_get::<GetMembers>))
            .route("/search", web::get().to(route_get_apub::<Search>))
            .route("/settings", web::get().to(route_get::<GetUserSettings>))
            .route("/settings", web::put().to(route_post::<SaveUserSettings>))
            .route("/messages", web::get().to(route_get_crud::<GetMessages>))*/
            // resolve federated objects (object => post, person, board or comment)
            /*.route(
                "/resolve_object",
                web::get().to(route_get_apub::<ResolveObject>),
            )*/
            /*.route(
                "/password_reset",
                web::post().to(route_post::<PasswordResetRequest>),
            )*/
            /*.route(
                "/password_reset/{reset_token}",
                web::post().to(route_post::<ExecutePasswordReset>),
            )*/
            // Get Federated Instances
            /*.service(
                web::scope("/federated_instances")
                    .wrap(rate_limit.message())
                    .route("", web::get().to(route_get::<GetFederatedInstances>)),
            )*/
            // Validate Site Invite
            /*.route(
                "/validate_invite/{invite_token}",
                web::post().to(route_post::<ValidateSiteInvite>),
            )*/
            // File Upload / Deletion
            /*.service(
                web::scope("/file")
                    .route("/upload", web::put().to(upload_file::<Multipart>))
                    .route("/{file_name}", web::delete().to(route_post::<DeleteFile>)),
            )*/
            // Authenticate
            /*.service(
                web::scope("/auth")
                    //.wrap(rate_limit.message())
                    .route("/login", web::post().to(route_post::<Login>))
                    .route("/signup", web::post().to(route_post_crud::<Register>))
                    // Delete Account
                    .route(
                        "/delete_account",
                        web::post().to(route_post_crud::<DeleteAccount>),
                    ),
            )*/
            // User
            /*.service(web::scope("/names").route("", web::get().to(route_get::<SearchNames>)))
            .service(
                web::scope("/user")
                    .route("", web::get().to(route_get_apub::<GetPersonDetails>))
                    .route("/{username}", web::get().to(route_get::<Profile>))
                    .route(
                        "/verify_email/{token}",
                        web::post().to(route_post::<VerifyEmail>),
                    ),
            )*/
            // Notifications
            /*.service(
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
            )*/
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
            /*.service(
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
            )*/
            // Post
            /*.service(
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
            )*/
            // Comment
            /*.service(
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
            )*/
            // Mod Actions
            /*.service(
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
            )*/
            // Admin Actions
            /*.service(
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
            ),*/
    );
}*/

pub fn graphql_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v2/graphql", web::post().to(perform_graphql));
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

async fn perform_graphql(
    context: web::Data<TinyBoardsContext>,
    graphql_request: GraphQLRequest,
    http_request: HttpRequest,
) -> Result<GraphQLResponse> {
    let auth_header = get_auth(&http_request);

    let logged_in_user =
        get_user_from_header_opt(context.pool(), context.master_key(), auth_header).await?;

    let my_person_id = match logged_in_user {
        Some(ref v) => v.id,
        None => -1,
    };

    Ok(context
        .schema()
        .execute(
            graphql_request
                .into_inner()
                .data(LoggedInUser::from(logged_in_user))
                .data(MasterKey::from(context.master_key().jwt.clone()))
                .data(GQLSettings::from(context.settings()))
                .data(context.pool().clone())
                .data(DataLoader::new(
                    PostgresLoader::new(context.pool(), my_person_id),
                    tokio::spawn,
                )),
        )
        .await
        .into())
}

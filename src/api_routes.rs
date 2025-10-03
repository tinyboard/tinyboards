use actix_web::*;
use async_graphql::dataloader::DataLoader;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
//use tinyboards_api::{Perform, PerformUpload};
use tinyboards_api::{context::TinyBoardsContext, utils::auth::get_user_from_header_opt};
use tinyboards_api::{LoggedInUser, MasterKey, PostgresLoader, Settings as GQLSettings};
use crate::media_handler;

pub fn graphql_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v2/graphql", web::post().to(perform_graphql));
}

pub fn media_files_config(cfg: &mut web::ServiceConfig) {
    // Serve media files through OpenDAL storage backend (works for all backends: fs, s3, azure, gcs)
    cfg.route("/media/{filename:.*}", web::get().to(media_handler::serve_media));
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

    let my_user_id = match logged_in_user {
        Some(ref v) => v.id,
        None => -1,
    };

    Ok(context
        .schema()
        .execute(
            graphql_request
                .into_inner()
                .data(LoggedInUser::from(logged_in_user))
                .data(MasterKey::from(context.master_key().jwt_secret.clone()))
                .data(GQLSettings::from(context.settings()))
                .data(context.pool().clone())
                .data(DataLoader::new(
                    PostgresLoader::new(context.pool(), my_user_id),
                    tokio::spawn,
                )),
        )
        .await
        .into())
}

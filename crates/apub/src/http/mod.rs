use crate::{
    activity_lists::SharedInboxActivities,
    fetcher::user_or_board::UserOrBoard,
    protocol::objects::tombstone::Tombstone,
    CONTEXT,
  };
use tinyboards_federation::{
    actix_web::inbox::receive_activity,
    config::Data,
    protocol::context::WithContext,
    FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use http::StatusCode;
use tinyboards_api_common::{data::TinyBoardsContext};
use tinyboards_db::models::apub::activity::Activity;
use tinyboards_utils::error::{TinyBoardsError, TinyBoardsResult};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

mod comment;
mod board;
mod person;
mod post;
pub mod routes;
pub mod site;

pub async fn shared_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<TinyBoardsContext>
) -> TinyBoardsResult<HttpResponse> {
    receive_activity::<SharedInboxActivities, UserOrBoard, TinyBoardsContext>(request, body, &data)
        .await
}

/// Convert the data to json and turn it into a HTTP Response with the correct
/// ActivityPub headers.
/// 
/// actix-web doesn't allow pretty-print for json so it needs to be done manually
fn create_apub_response<T>(data: &T) -> TinyBoardsResult<HttpResponse>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(&WithContext::new(data, CONTEXT.clone()))?;

    Ok(
        HttpResponse::Ok()
            .content_type(FEDERATION_CONTENT_TYPE)
            .content_type("application/activity+json")
            .body(json),
    )
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> TinyBoardsResult<HttpResponse> {
    let tombstone = Tombstone::new(id.into());
    let json = serde_json::to_string_pretty(&WithContext::new(tombstone, CONTEXT.deref().clone()))?;

    Ok(
        HttpResponse::Gone()
            .content_type(FEDERATION_CONTENT_TYPE)
            .status(StatusCode::GONE)
            .content_type("application/activity+json")
            .body(json),
    )
}

fn err_object_not_local() -> TinyBoardsError {
    TinyBoardsError::from_message(400, "object is not local, fetch it from original instance")
}

#[derive(Deserialize)]
pub struct ActivityQuery {
    type_: String,
    id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
pub(crate) async fn get_activity(
    info: web::Path<ActivityQuery>,
    context: web::Data<TinyBoardsContext>
) -> Result<HttpResponse, TinyBoardsError> {
    let settings = context.settings();
    let activity_id = Url::parse(&format!(
        "{}/activities/{}/{}",
        settings.get_protocol_and_hostname(),
        info.type_,
        info.id,
    ))?
    .into();
    let activity = Activity::read_from_apub_id(context.pool(), &activity_id).await?;

    let sensitive = activity.sensitive;
    if !activity.local {
        Err(err_object_not_local())
    } else if sensitive {
        Ok(HttpResponse::Forbidden().finish())
    } else {
        create_apub_response(&activity.data)
    }   
}
use crate::{
    insert_activity,
    objects::{person::ApubPerson, board::ApubBoard},
    CONTEXT,
};
use tinyboards_db_views::structs::{BoardPersonBanView, BoardView};
use tinyboards_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::public,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_db::models::board::boards::Board;
use serde::Serialize;
use std::ops::Deref;
use tracing::info;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod board;

/// Checks that the specified Url actually specifies a Person (by fetching it), and that the person has no site ban.
#[tracing::instrument(skip_all)]
async fn verify_person(
    person_id: &ObjectId<ApubPerson>,
    context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
    let person = person_id.dereference(context).await?;
    if person.is_banned {
        let err = anyhow!("Person {} is banned", person_id);
        return Err(TinyBoardsError::from_error_message(err, 403, "banned"));
    }
    Ok(())
}

/// Fetches the person and the board to verify their type, then checks if person is banned from site or board.
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_person_in_board(
    person_id: &ObjectId<ApubPerson>,
    board: &ApubBoard,
    context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
    let person = person_id.dereference(context).await?;
    if person.is_banned {
        return Err(TinyBoardsError::from_message(403, "Person is banned from site"));
    }
    let person_id = person.id;
    let board_id = board.id;
    let is_banned = BoardPersonBanView::get(context.pool(), person_id, board_id)
        .await
        .is_ok();
    if is_banned {
        return Err(TinyBoardsError::from_message(403, "Person is banned from board"));
    }
    Ok(())
}

/// Verify that mod action in the board was performed by a moderator
/// 
/// * `mod_id` = Apub ID of the mod or admin who performed the action
/// * `object_id` = Apub ID of the actor or object being moderated
/// * `board` = The board in which the moderation is taking place
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_mod_action(
    mod_id: &ObjectId<ApubPerson>,
    object_id: &Url,
    board_id: i32,
    context: &Data<TinyBoardsContext>
) -> Result<(), TinyBoardsError> {
    let moderator = mod_id.dereference(context).await?;
    let is_mod_or_admin = 
        BoardView::is_mod_or_admin(context.pool(), moderator.id.clone(), board_id.clone())
        .await;
    if is_mod_or_admin {
        return Ok(());
    }
    // mod action comes from the same instance as the moderated object, so it was presumably done by an instance admin
    if mod_id.inner().domain() == object_id.domain() {
        return Ok(());
    }
    
    Err(TinyBoardsError::from_message(403, "not a moderator or admin"))
}

pub(crate) fn verify_is_public(to: &[Url], cc: &[Url]) -> Result<(), TinyBoardsError> {
    if ![to, cc].iter().any(|set| set.contains(&public())) {
        return Err(TinyBoardsError::from_message(403, "object is not public"));
    }
    Ok(())
}

pub(crate) fn verify_board_matches<T>(
    a: &ObjectId<ApubBoard>,
    b: T,
) -> Result<(), TinyBoardsError>
where
    T: Into<ObjectId<ApubBoard>> 
{
    let b: ObjectId<ApubBoard> = b.into();
    if a != &b {
        return Err(TinyBoardsError::from_message(400, "invalid board"));
    }
    Ok(())
}

pub(crate) fn check_board_deleted(board: &Board) -> Result<(), TinyBoardsError> {
    if board.is_deleted {
        Err(TinyBoardsError::from_message(400, "New post or comment can't be made in a deleted board."))
    } else {
        Ok(())
    }
}

/// Generate a unique ID for an activity
/// Ex: http(s)://example.com/receive/create/UUID
fn generate_activity_id<T>(kind: T, protocol_and_hostname: &str) -> Result<Url, ParseError>
where
    T: ToString
{
    let id = format!(
        "{}/activities/{}/{}",
        protocol_and_hostname,
        kind.to_string().to_lowercase(),
        Uuid::new_v4(),
    );
    Url::parse(&id)
}

#[tracing::instrument(skip_all)]
async fn send_tinyboards_activity<Activity, ActorT>(
    data: &Data<TinyBoardsContext>,
    activity: Activity,
    actor: &ActorT,
    inbox: Vec<Url>,
    sensitive: bool,
) -> Result<(), TinyBoardsError> 
where
    Activity: ActivityHandler + Serialize + Send + Sync + Clone,
    ActorT: Actor,
    Activity: ActivityHandler<Error = TinyBoardsError>,
{
    info!("Sending activity {}", activity.id().to_string());
    let activity = WithContext::new(activity, CONTEXT.deref().clone());

    // insert activity into local db
    insert_activity(activity.id(), &activity, true, sensitive, data).await?;
    // send the activity!
    send_activity(activity, actor, inbox, data).await?;

    Ok(())
}
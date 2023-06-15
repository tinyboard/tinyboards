use crate::{
    insert_activity,
    objects::{/*board::ApubBoard,*/ person::ApubPerson},
    CONTEXT,
};
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

// #[tracing::instrument(skip_all)]
// pub(crate) async fn verify_person_in_board(
//     person_id: &ObjectId<ApubPerson>,
//     board: &ApubBoard,
//     context: &Data<TinyBoardsContext>,
// ) -> Result<(), TinyBoardsError> {

// }
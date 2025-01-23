use crate::{
    activities::verify_board_matches,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{InBoard, objects::group::Group},
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UpdateType,
    protocol::helpers::deserialize_one_or_many,
};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use serde::{Deserialize, Serialize};
use url::Url;

/// This activity is received from a remote board mod, and updates the description or other
/// fields of a local board.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoard {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Box<Group>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UpdateType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubBoard>>,
}

#[async_trait::async_trait]
impl InBoard for UpdateBoard {
  async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
    let board: ApubBoard = self.object.id.clone().dereference(context).await?;
    if let Some(audience) = &self.audience {
      verify_board_matches(audience, board.actor_id.clone())?;
    }
    Ok(board)
  }
}
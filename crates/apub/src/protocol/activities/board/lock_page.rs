use crate::{
    activities::verify_board_matches,
    objects::{board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::InBoard,
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UndoType,
    protocol::helpers::deserialize_one_or_many,
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{models::board::boards::Board, traits::Crud};
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use strum_macros::Display;
  use url::Url;
  
  #[derive(Clone, Debug, Deserialize, Serialize, Display)]
  pub enum LockType {
    Lock,
  }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockPage {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: ObjectId<ApubPost>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: LockType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubBoard>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoLockPage {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: LockPage,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubBoard>>,
}

#[async_trait::async_trait]
impl InBoard for LockPage {
  async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
    let post = self.object.dereference(context).await?;
    let board = Board::read(context.pool(), post.board_id).await?;
    if let Some(audience) = &self.audience {
      verify_board_matches(audience, board.actor_id.clone())?;
    }
    Ok(board.into())
  }
}

#[async_trait::async_trait]
impl InBoard for UndoLockPage {
  async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
    let board = self.object.board(context).await?;
    if let Some(audience) = &self.audience {
      verify_board_matches(audience, board.actor_id.clone())?;
    }
    Ok(board)
  }
}
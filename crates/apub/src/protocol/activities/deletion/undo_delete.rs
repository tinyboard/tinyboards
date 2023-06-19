use crate::{
    activities::verify_board_matches,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{activities::deletion::delete::Delete, InBoard},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UndoType,
    protocol::helpers::deserialize_one_or_many,
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use serde_with::skip_serializing_none;
  use url::Url;
  
  #[skip_serializing_none]
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct UndoDelete {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: Delete,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>,
    #[serde(deserialize_with = "deserialize_one_or_many", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) cc: Vec<Url>,
  }
  
  #[async_trait::async_trait]
  impl InBoard for UndoDelete {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let board = self.object.board(context).await?;
      if let Some(audience) = &self.audience {
        verify_board_matches(audience, board.actor_id.clone())?;
      }
      Ok(board)
    }
  }
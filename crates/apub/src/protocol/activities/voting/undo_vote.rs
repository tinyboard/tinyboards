use crate::{
    activities::verify_board_matches,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{activities::voting::vote::Vote, InBoard},
  };
  use tinyboards_federation::{config::Data, fetch::object_id::ObjectId, kinds::activity::UndoType};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use url::Url;
  
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct UndoVote {
    pub(crate) actor: ObjectId<ApubPerson>,
    pub(crate) object: Vote,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>,
  }
  
  #[async_trait::async_trait]
  impl InBoard for UndoVote {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let board = self.object.community(context).await?;
      if let Some(audience) = &self.audience {
        verify_board_matches(audience, board.actor_id.clone())?;
      }
      Ok(board)
    }
  }
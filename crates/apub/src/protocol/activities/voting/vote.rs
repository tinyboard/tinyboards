use crate::{
    activities::verify_board_matches,
    fetcher::post_or_comment::PostOrComment,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::InBoard,
  };
  use tinyboards_federation::{config::Data, fetch::object_id::ObjectId};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use std::convert::TryFrom;
  use strum_macros::Display;
  use url::Url;
  
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Vote {
    pub(crate) actor: ObjectId<ApubPerson>,
    pub(crate) object: ObjectId<PostOrComment>,
    #[serde(rename = "type")]
    pub(crate) kind: VoteType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>,
  }
  
  #[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq, Eq)]
  pub enum VoteType {
    Like,
    Dislike,
  }
  
  impl TryFrom<i16> for VoteType {
    type Error = TinyBoardsError;
  
    fn try_from(value: i16) -> Result<Self, Self::Error> {
      match value {
        1 => Ok(VoteType::Like),
        -1 => Ok(VoteType::Dislike),
        _ => Err(TinyBoardsError::from_message(400, "invalid vote value")),
      }
    }
  }
  
  impl From<&VoteType> for i16 {
    fn from(value: &VoteType) -> i16 {
      match value {
        VoteType::Like => 1,
        VoteType::Dislike => -1,
      }
    }
  }
  
  #[async_trait::async_trait]
  impl InBoard for Vote {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let board = self
        .object
        .dereference(context)
        .await?
        .board(context)
        .await?;
      if let Some(audience) = &self.audience {
        verify_board_matches(audience, board.actor_id.clone())?;
      }
      Ok(board)
    }
  }
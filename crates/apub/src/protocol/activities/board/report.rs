use crate::{
    activities::verify_board_matches,
    fetcher::post_or_comment::PostOrComment,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::InBoard,
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FlagType,
    protocol::helpers::deserialize_one,
};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one")]
    pub(crate) to: [ObjectId<ApubBoard>; 1],
    pub(crate) object: ObjectId<PostOrComment>,
    pub(crate) summary: String,
    #[serde(rename = "type")]
    pub(crate) kind: FlagType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>,
}

#[async_trait::async_trait]
impl InBoard for Report {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
        let board = self.to[0].dereference(context).await?;
        if let Some(audience) = &self.audience {
          verify_board_matches(audience, board.actor_id.clone())?;
        }
        Ok(board)
      }
}
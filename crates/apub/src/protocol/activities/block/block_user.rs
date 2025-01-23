use crate::{
    activities::{block::SiteOrBoard, verify_board_matches},
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::InBoard,
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::BlockType,
    protocol::helpers::deserialize_one_or_many,
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUser {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) cc: Vec<Url>,
    pub(crate) target: ObjectId<SiteOrBoard>,
    #[serde(rename = "type")]
    pub(crate) kind: BlockType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>, 
    /// TODO: send a separate Delete activity instead of this remove_data field
    pub(crate) remove_data: Option<bool>,
    /// block reason, written to mod log
    pub(crate) summary: Option<String>,
    pub(crate) expires: Option<DateTime<FixedOffset>>,  
}

#[async_trait::async_trait]
impl InBoard for BlockUser {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
        let target = self.target.dereference(context).await?;
        let board = match target {
            SiteOrBoard::Board(b) => b,
            SiteOrBoard::Site(_) => return Err(anyhow!("activity is not in board").into()),
        };
        if let Some(audience) = &self.audience {
            verify_board_matches(audience, board.actor_id.clone())?;
        }
        Ok(board)
    }
}
use crate::{
    activities::{deletion::DeletableObjects, verify_board_matches},
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{objects::tombstone::Tombstone, IdOrNestedObject, InBoard},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::DeleteType,
    protocol::helpers::deserialize_one_or_many,
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{board::boards::Board, post::posts::Post},
    traits::Crud,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use serde_with::skip_serializing_none;
  use url::Url;
  
  #[skip_serializing_none]
  #[derive(Clone, Debug, Deserialize, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Delete {
    pub(crate) actor: ObjectId<ApubPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: IdOrNestedObject<Tombstone>,
    #[serde(rename = "type")]
    pub(crate) kind: DeleteType,
    pub(crate) id: Url,
    pub(crate) audience: Option<ObjectId<ApubBoard>>,
  
    #[serde(deserialize_with = "deserialize_one_or_many")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) cc: Vec<Url>,
    /// If summary is present, this is a mod action (Remove in Tinyboards terms). Otherwise, its a user
    /// deleting their own content.
    pub(crate) summary: Option<String>,
  }
  
  #[async_trait::async_trait]
  impl InBoard for Delete {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let board_id = match DeletableObjects::read_from_db(self.object.id(), context).await? {
        DeletableObjects::Board(b) => b.id,
        DeletableObjects::Comment(c) => {
          let post = Post::read(context.pool(), c.post_id).await?;
          post.board_id
        }
        DeletableObjects::Post(p) => p.board_id
      };
      let board = Board::read(context.pool(), board_id).await?;
      if let Some(audience) = &self.audience {
        verify_board_matches(audience, board.actor_id.clone())?;
      }
      Ok(board.into())
    }
  }
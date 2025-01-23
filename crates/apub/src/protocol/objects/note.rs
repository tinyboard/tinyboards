use crate::{
    activities::verify_board_matches,
    fetcher::post_or_comment::PostOrComment,
    mentions::MentionOrValue,
    objects::{comment::ApubComment, board::ApubBoard, person::ApubPerson, post::ApubPost},
    protocol::{objects::LanguageTag, InBoard, Source},
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::object::NoteType,
    protocol::{
      helpers::{deserialize_one_or_many, deserialize_skip_error},
      values::MediaTypeMarkdownOrHtml,
    },
  };
  use chrono::{DateTime, FixedOffset};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{
        board::boards::Board, 
        post::posts::Post,
    },
    traits::Crud,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use serde::{Deserialize, Serialize};
  use serde_with::skip_serializing_none;
  use std::ops::Deref;
  use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  pub(crate) r#type: NoteType,
  pub(crate) id: ObjectId<ApubComment>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: String,
  pub(crate) in_reply_to: ObjectId<PostOrComment>,

  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  #[serde(default)]
  pub(crate) tag: Vec<MentionOrValue>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubBoard>>,
}

impl Note {
    pub(crate) async fn get_parents(
      &self,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(ApubPost, Option<ApubComment>), TinyBoardsError> {
      // Fetch parent comment chain in a box, otherwise it can cause a stack overflow.
      let parent = Box::pin(self.in_reply_to.dereference(context).await?);
      match parent.deref() {
        PostOrComment::Post(p) => Ok((p.clone(), None)),
        PostOrComment::Comment(c) => {
          let post_id = c.post_id;
          let post = Post::read(context.pool(), post_id).await?;
          Ok((post.into(), Some(c.clone())))
        }
      }
    }
  }
  
  #[async_trait::async_trait]
  impl InBoard for Note {
    async fn board(&self, context: &Data<TinyBoardsContext>) -> Result<ApubBoard, TinyBoardsError> {
      let (post, _) = self.get_parents(context).await?;
      let board = Board::read(context.pool(), post.board_id).await?;
      if let Some(audience) = &self.audience {
        verify_board_matches(audience, board.actor_id.clone())?;
      }
      Ok(board.into())
    }
  }
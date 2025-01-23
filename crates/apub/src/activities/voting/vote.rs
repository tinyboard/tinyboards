use crate::{
    activities::{
      generate_activity_id,
      verify_person_in_board,
      voting::{vote_comment, vote_post},
    },
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{
      activities::voting::vote::{Vote, VoteType},
      InBoard,
    },
    fetcher::post_or_comment::PostOrComment,
};
use tinyboards_federation::{
config::Data,
fetch::object_id::ObjectId,
traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_db::models::site::local_site::LocalSite;
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

impl Vote {
    pub(in crate::activities::voting) fn new(
      object_id: ObjectId<PostOrComment>,
      actor: &ApubPerson,
      board: &ApubBoard,
      kind: VoteType,
      context: &Data<TinyBoardsContext>,
    ) -> Result<Vote, TinyBoardsError> {
      Ok(Vote {
        actor: actor.id().into(),
        object: object_id,
        kind: kind.clone(),
        id: generate_activity_id(kind, &context.settings().get_protocol_and_hostname())?,
        audience: Some(board.id().into()),
      })
    }
  }

#[async_trait::async_trait]
impl ActivityHandler for Vote {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    let board = self.board(context).await?;
    verify_person_in_board(&self.actor, &board, context).await?;
    let enable_downvotes = LocalSite::read(context.pool())
      .await
      .map(|l| l.enable_downvotes)
      .unwrap_or(true);
    if self.kind == VoteType::Dislike && !enable_downvotes {
      return Err(anyhow!("downvotes disabled").into());
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, true, context).await?;
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, &p, context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, &c, context).await,
    }
  }
}
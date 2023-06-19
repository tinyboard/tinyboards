use crate::{
    activities::{
      generate_activity_id,
      verify_person_in_board,
      voting::{undo_vote_comment, undo_vote_post},
    },
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{
      activities::voting::{undo_vote::UndoVote, vote::Vote},
      InBoard,
    },
    fetcher::post_or_comment::PostOrComment,
  };
  use tinyboards_federation::{
    config::Data,
    kinds::activity::UndoType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;
  
  impl UndoVote {
    pub(in crate::activities::voting) fn new(
      vote: Vote,
      actor: &ApubPerson,
      board: &ApubBoard,
      context: &Data<TinyBoardsContext>,
    ) -> Result<Self, TinyBoardsError> {
      Ok(UndoVote {
        actor: actor.id().into(),
        object: vote,
        kind: UndoType::Undo,
        id: generate_activity_id(
          UndoType::Undo,
          &context.settings().get_protocol_and_hostname(),
        )?,
        audience: Some(board.id().into()),
      })
    }
  }
  
  #[async_trait::async_trait]
  impl ActivityHandler for UndoVote {
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
      verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
      self.object.verify(context).await?;
      Ok(())
    }
  
    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
      insert_activity(&self.id, &self, false, true, context).await?;
      let actor = self.actor.dereference(context).await?;
      let object = self.object.object.dereference(context).await?;
      match object {
        PostOrComment::Post(p) => undo_vote_post(actor, &p, context).await,
        PostOrComment::Comment(c) => undo_vote_comment(actor, &c, context).await,
      }
    }
  }
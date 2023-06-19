use crate::{
    activities::{
      deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
      generate_activity_id,
    },
    insert_activity,
    objects::person::ApubPerson,
    protocol::activities::deletion::{delete::Delete, undo_delete::UndoDelete},
  };
  use tinyboards_federation::{config::Data, kinds::activity::UndoType, traits::ActivityHandler};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{
      comment::comments::{Comment},
      board::boards::{Board},
      moderator::mod_actions::{
        ModRemoveComment,
        ModRemoveCommentForm,
        ModRemoveBoard,
        ModRemoveBoardForm,
        ModRemovePost,
        ModRemovePostForm,
      },
      post::posts::{Post},
    },
    traits::Crud,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for UndoDelete {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    self.object.verify(data).await?;
    verify_delete_activity(&self.object, self.object.summary.is_some(), data).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, false, context).await?;
    if self.object.summary.is_some() {
      UndoDelete::receive_undo_remove_action(
        &self.actor.dereference(context).await?,
        self.object.object.id(),
        context,
      )
      .await
    } else {
      receive_delete_action(self.object.object.id(), &self.actor, false, context).await
    }
  }
}

impl UndoDelete {
    #[tracing::instrument(skip_all)]
    pub(in crate::activities::deletion) fn new(
      actor: &ApubPerson,
      object: DeletableObjects,
      to: Url,
      board: Option<&Board>,
      summary: Option<String>,
      context: &Data<TinyBoardsContext>,
    ) -> Result<UndoDelete, TinyBoardsError> {
      let object = Delete::new(actor, object, to.clone(), board, summary, context)?;
  
      let id = generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?;
      let cc: Option<Url> = board.map(|b| b.actor_id.clone().into());
      Ok(UndoDelete {
        actor: actor.actor_id.clone().into(),
        to: vec![to],
        object,
        cc: cc.into_iter().collect(),
        kind: UndoType::Undo,
        id,
        audience: board.map(|b| b.actor_id.clone().into()),
      })
    }
  
    #[tracing::instrument(skip_all)]
    pub(in crate::activities) async fn receive_undo_remove_action(
      actor: &ApubPerson,
      object: &Url,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      match DeletableObjects::read_from_db(object, context).await? {
        DeletableObjects::Board(board) => {
          if board.local {
            return Err(TinyBoardsError::from_message(
              403, "only local admin can restore board",
            ));
          }
          let form = ModRemoveBoardForm {
            mod_person_id: actor.id,
            board_id: board.id,
            removed: Some(Some(false)),
            reason: None,
          };
          ModRemoveBoard::create(context.pool(), &form).await?;
          Board::update_removed(context.pool(), board.id, false).await?;
        },
        DeletableObjects::Post(post) => {
          let form = ModRemovePostForm {
            mod_person_id: actor.id,
            post_id: post.id,
            removed: Some(Some(false)),
            reason: None,
          };
          ModRemovePost::create(context.pool(), &form).await?;
          Post::update_removed(context.pool(), post.id, false).await?;
        },
        DeletableObjects::Comment(comment) => {
          let form = ModRemoveCommentForm {
            mod_person_id: actor.id,
            comment_id: comment.id,
            removed: Some(Some(false)),
            reason: None,
          };
          ModRemoveComment::create(context.pool(), &form).await?;
          Comment::update_removed(context.pool(), comment.id, false).await?;
        },
      }
      Ok(())
    }
  }
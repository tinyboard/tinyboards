use crate::{
    activities::{
      deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
      generate_activity_id,
    },
    insert_activity,
    objects::person::ApubPerson,
    protocol::{activities::deletion::delete::Delete, IdOrNestedObject},
  };
  use tinyboards_federation::{config::Data, kinds::activity::DeleteType, traits::ActivityHandler};
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
impl ActivityHandler for Delete {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
    verify_delete_activity(self, self.summary.is_some(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, false, context).await?;
    if let Some(reason) = self.summary {
      // We set reason to empty string if it doesn't exist, to distinguish between delete and
      // remove. Here we change it back to option, so we don't write it to db.
      let reason = if reason.is_empty() {
        None
      } else {
        Some(reason)
      };
      receive_remove_action(
        &self.actor.dereference(context).await?,
        self.object.id(),
        reason,
        context,
      )
      .await
    } else {
      receive_delete_action(self.object.id(), &self.actor, true, context).await
    }
  }
}

impl Delete {
  pub(in crate::activities::deletion) fn new(
    actor: &ApubPerson,
    object: DeletableObjects,
    to: Url,
    board: Option<&Board>,
    summary: Option<String>,
    context: &Data<TinyBoardsContext>,
  ) -> Result<Delete, TinyBoardsError> {
    let id = generate_activity_id(
      DeleteType::Delete,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let cc: Option<Url> = board.map(|b| b.actor_id.clone().into());
    Ok(Delete {
      actor: actor.actor_id.clone().into(),
      to: vec![to],
      object: IdOrNestedObject::Id(object.id()),
      cc: cc.into_iter().collect(),
      kind: DeleteType::Delete,
      summary,
      id,
      audience: board.map(|b| b.actor_id.clone().into()),
    })
  }
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn receive_remove_action(
  actor: &ApubPerson,
  object: &Url,
  reason: Option<String>,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Board(board) => {
      if board.local {
        return Err(TinyBoardsError::from_message(
          403, "only local admin can remove board",
        ));
      }
      let form = ModRemoveBoardForm {
        mod_person_id: actor.id,
        board_id: board.id,
        removed: Some(Some(true)),
        reason: Some(reason),
      };
      ModRemoveBoard::create(context.pool(), &form).await?;
      Board::update_removed(context.pool(), board.id, true)?;
    },
    DeletableObjects::Post(post) => {
      let form = ModRemovePostForm {
        mod_person_id: actor.id,
        post_id: post.id,
        removed: Some(Some(true)),
        reason: Some(reason),
      };
      ModRemovePost::create(context.pool(), &form).await?;
      Post::update_removed(context.pool(), post.id, true).await?;
    },
    DeletableObjects::Comment(comment) => {
      let form = ModRemoveCommentForm {
        mod_person_id: actor.id,
        comment_id: comment.id,
        removed: Some(Some(true)),
        reason: Some(reason),
      };
      ModRemoveComment::create(context.pool(), &form).await?;
      Comment::update_removed(context.pool(), comment.id, true).await?;
    },
  }
  Ok(())
}
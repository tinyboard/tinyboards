use crate::{
    activities::{
      board::send_activity_in_board,
      verify_is_public,
      verify_mod_action,
      verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    objects::{
      comment::ApubComment,
      board::ApubBoard,
      person::ApubPerson,
      post::ApubPost,
    },
    protocol::{
      activities::deletion::{delete::Delete, undo_delete::UndoDelete},
      InBoard,
    },
    SendActivity,
  };
use tinyboards_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::public,
  protocol::verification::verify_domains_match,
  traits::{Actor, Object},
};
use tinyboards_api_common::{
  comment::{CommentResponse, DeleteComment, RemoveComment},
  board::{BoardResponse, DeleteBoard},
  data::TinyBoardsContext,
  post::{DeletePost, PostResponse, RemovePost},
  utils::{require_user},
};
use tinyboards_db::{
  models::{
      comment::comments::{Comment},
      board::boards::{Board},
      person::person::Person,
      post::posts::{Post},
  },
  traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;
use std::ops::Deref;
use url::Url;

pub mod delete;
pub mod delete_user;
pub mod undo_delete;

#[async_trait::async_trait]
impl SendActivity for DeletePost {
  type Response = PostResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    let board = Board::read(context.pool(), response.post_view.board.id).await?;
    let deletable = DeletableObjects::Post(response.post_view.post.clone().into());
    send_apub_delete_in_board(
      view.person,
      board,
      deletable,
      None,
      request.deleted,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for RemovePost {
  type Response = PostResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth)
      .await
      .unwrap()?;
    let board = Board::read(context.pool(), response.post_view.board.id).await?;
    let deletable = DeletableObjects::Post(response.post_view.post.clone().into());
    send_apub_delete_in_board(
      view.person,
      board,
      deletable,
      request.reason.clone().or_else(|| Some(String::new())),
      request.removed,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for DeleteComment {
  type Response = CommentResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    _auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let board_id = response.comment_view.board.id;
    let board = Board::read(context.pool(), board_id).await?;
    let person = Person::read(context.pool(), response.comment_view.creator.clone().unwrap().id).await?;
    let deletable = DeletableObjects::Comment(response.comment_view.comment.clone().into());
    send_apub_delete_in_board(person, board, deletable, None, request.deleted, context)
      .await
  }
}

#[async_trait::async_trait]
impl SendActivity for RemoveComment {
  type Response = CommentResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    let comment = Comment::read(context.pool(), request.comment_id).await?;
    let board = Board::read(context.pool(), response.comment_view.board.id).await?;
    let deletable = DeletableObjects::Comment(comment.into());
    send_apub_delete_in_board(
      view.person,
      board,
      deletable,
      request.reason.clone().or_else(|| Some(String::new())),
      request.removed,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for DeleteBoard {
  type Response = BoardResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth)
      .await
      .unwrap()?;
    let board = Board::read(context.pool(), response.board_view.board.id).await?;
    let deletable = DeletableObjects::Board(board.clone().into());
    send_apub_delete_in_board(
      view.person,
      board,
      deletable,
      None,
      response.board_view.board.is_deleted,
      context,
    )
    .await
  }
}

pub enum DeletableObjects {
  Board(ApubBoard),
  Comment(ApubComment),
  Post(ApubPost),
}

impl DeletableObjects {
  #[tracing::instrument(skip_all)]
  pub(crate) async fn read_from_db(
    ap_id: &Url,
    context: &Data<TinyBoardsContext>
  ) -> Result<DeletableObjects, TinyBoardsError> {
    if let Some(b) = ApubBoard::read_from_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Board(b));
    }
    if let Some(p) = ApubPost::read_from_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Post(p));
    }
    if let Some(c) = ApubComment::read_from_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Comment(c));
    }
    Err(diesel::NotFound.into())
  }

  pub(crate) fn id(&self) -> Url {
    match self {
      DeletableObjects::Board(b) => b.id(),
      DeletableObjects::Comment(c) => c.ap_id.clone().unwrap().into(),
      DeletableObjects::Post(p) => p.ap_id.clone().unwrap().into(),
    }
  }
}


/// Parameter `reason` being set indicates that this is a removal by a mod. If its unset, this
/// action was done by a normal user.
#[tracing::instrument(skip_all)]
async fn send_apub_delete_in_board(
  actor: Person,
  board: Board,
  object: DeletableObjects,
  reason: Option<String>,
  deleted: bool,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let actor = ApubPerson::from(actor);
  let is_mod_action = reason.is_some();
  let activity = if deleted {
    let delete = Delete::new(&actor, object, public(), Some(&board), reason, context)?;
    AnnouncableActivities::Delete(delete)
  } else {
    let undo = UndoDelete::new(&actor, object, public(), Some(&board), reason, context)?;
    AnnouncableActivities::UndoDelete(undo)
  };
  send_activity_in_board(
    activity,
    &actor,
    &board.into(),
    vec![],
    is_mod_action,
    context,
  )
  .await
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn verify_delete_activity(
  activity: &Delete,
  is_mod_action: bool,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let object = DeletableObjects::read_from_db(activity.object.id(), context).await?;
  match object {
    DeletableObjects::Board(board) => {
      verify_is_public(&activity.to, &[])?;
      if board.local {
        // can only do this check for local board, in remote case it would try to fetch the
        // deleted board (which fails)
        verify_person_in_board(&activity.actor, &board, context).await?;
      }
      // board deletion is always a mod (or admin) action
      verify_mod_action(&activity.actor, activity.object.id(), board.id, context).await?;
    },
    DeletableObjects::Post(p) => {
      verify_is_public(&activity.to, &[])?;
      verify_delete_post_or_comment(
        &activity.actor,
        &p.ap_id.clone().unwrap().into(),
        &activity.board(context).await?,
        is_mod_action,
        context,
      )
      .await?;
    },
    DeletableObjects::Comment(c) => {
      verify_is_public(&activity.to, &[])?;
      verify_delete_post_or_comment(
        &activity.actor,
        &c.ap_id.clone().unwrap().into(),
        &activity.board(context).await?,
        is_mod_action,
        context,
      )
      .await?;
    }
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn verify_delete_post_or_comment(
  actor: &ObjectId<ApubPerson>,
  object_id: &Url,
  board: &ApubBoard,
  is_mod_action: bool,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  verify_person_in_board(actor, board, context).await?;
  if is_mod_action {
    verify_mod_action(actor, object_id, board.id, context).await?;
  } else {
    // domain of post ap_id and post.creator ap_id are identical, so we just check the former
    verify_domains_match(actor.inner(), object_id)?;
  }
  Ok(())
}

/// Write deletion or restoring of an object to the database, and send websocket message.
#[tracing::instrument(skip_all)]
async fn receive_delete_action(
  object: &Url,
  actor: &ObjectId<ApubPerson>,
  deleted: bool,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Board(board) => {
      if board.local {
        let mod_: Person = actor.dereference(context).await?.deref().clone();
        let object = DeletableObjects::Board(board.clone());
        let b: Board = board.deref().deref().clone();
        send_apub_delete_in_board(mod_, b, object, None, true, context).await?;
      }
      Board::update_deleted(context.pool(), board.id, deleted).await?;
    },
    DeletableObjects::Post(post) => {
      if deleted != post.is_deleted {
        Post::update_deleted(context.pool(), post.id, deleted).await?;
      }
    },
    DeletableObjects::Comment(comment) => {
      if deleted != comment.is_deleted {
        Comment::update_deleted(context.pool(), comment.id, deleted).await?;
      }
    },
  }
  Ok(())
}
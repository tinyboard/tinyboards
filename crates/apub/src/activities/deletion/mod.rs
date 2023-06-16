use crate::{
    activities::{
      board::send_activity_in_board,
      send_tinyboards_activity,
      verify_is_public,
      verify_mod_action,
      verify_person,
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
  comment::{CommentResponse, DeleteComment},
  board::{BoardResponse, DeleteBoard},
  data::TinyBoardsContext,
  post::{DeletePost, PostResponse},
  utils::get_local_user_view_from_jwt,
};
use tinyboards_db::{
  models::{
      comment::comments::{Comment, CommentForm},
      board::boards::{Board, BoardForm},
      person::person::Person,
      post::posts::{Post, PostForm},
  },
  traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;
use std::ops::Deref;
use url::Url;

pub mod delete;
pub mod delete_user;
pub mod undo_delete;

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
      DeletableObjects::Comment(c) => c.ap_id.clone().into(),
      DeletableObjects::Post(p) => p.ap_id.clone().into(),
    }
  }
}
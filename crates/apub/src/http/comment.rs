use crate::{
    http::{create_apub_response, create_apub_tombstone_response, err_object_not_local},
    objects::comment::ApubComment,
  };
  use tinyboards_federation::{config::Data, traits::Object};
  use actix_web::{web::Path, HttpResponse};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{models::comment::comments::Comment, traits::Crud};
  use tinyboards_utils::error::TinyBoardsError;
  use serde::Deserialize;
  
  #[derive(Deserialize)]
  pub(crate) struct CommentQuery {
    comment_id: String,
  }
  
  /// Return the ActivityPub json representation of a local comment over HTTP.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_comment(
    info: Path<CommentQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let id = info.comment_id.parse::<i32>()?;
    let comment: ApubComment = Comment::read(context.pool(), id).await?.into();
    if !comment.local {
      return Err(err_object_not_local());
    }
  
    if !comment.is_deleted && !comment.is_removed {
      create_apub_response(&comment.into_json(&context).await?)
    } else {
      create_apub_tombstone_response(comment.ap_id.clone().unwrap())
    }
}
use crate::{
    http::{create_apub_response, create_apub_tombstone_response, err_object_not_local},
    objects::post::ApubPost,
  };
  use tinyboards_federation::{config::Data, traits::Object};
  use actix_web::{web, HttpResponse};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{models::post::posts::Post, traits::Crud};
  use tinyboards_utils::error::TinyBoardsError;
  use serde::Deserialize;
  
  #[derive(Deserialize)]
  pub(crate) struct PostQuery {
    post_id: String,
  }
  
  /// Return the ActivityPub json representation of a local post over HTTP.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_post(
    info: web::Path<PostQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let id = info.post_id.parse::<i32>()?;
    let post: ApubPost = Post::read(context.pool(), id).await?.into();
    if !post.local {
      return Err(err_object_not_local());
    }
  
    if !post.is_deleted && !post.is_removed {
      create_apub_response(&post.into_json(&context).await?)
    } else {
      create_apub_tombstone_response(post.ap_id.clone().unwrap())
    }
}
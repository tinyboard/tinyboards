use crate::{
    api::{listing_type_with_default, PerformApub},
    fetcher::resolve_actor_identifier,
    objects::board::ApubBoard,
  };
  use tinyboards_federation::config::Data;
  use tinyboards_api_common::{
    comment::{GetComments, GetCommentsResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, require_user},
  };
  use tinyboards_db::{
    models::{board::boards::Board, site::local_site::LocalSite},
    traits::Crud,
  };
  use tinyboards_db_views::comment_view::CommentQuery;
  use tinyboards_utils::error::TinyBoardsError;
  
  #[async_trait::async_trait]
  impl PerformApub for GetComments {
    type Response = GetCommentsResponse;
  
    #[tracing::instrument(skip(context))]
    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<GetCommentsResponse, TinyBoardsError> {
      let data: &GetComments = self;
      let local_user_view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
      let local_site = LocalSite::read(context.pool()).await?;
      check_private_instance(&Some(local_user_view.local_user.clone()), context.pool()).await?;
  
      let board_id = if let Some(name) = &data.board_name {
        resolve_actor_identifier::<ApubBoard, Board>(name, context, &None, true)
          .await
          .ok()
          .map(|b| b.id)
      } else {
        data.board_id
      };
      let sort = data.sort;
      let saved_only = data.saved_only;
      let page = data.page;
      let limit = data.limit;
      let parent_id = data.parent_id;
      let listing_type = listing_type_with_default(data.type_, &local_site, board_id)?;
      let post_id = data.post_id;
      let local_user = local_user_view.local_user.clone();

      let resp = CommentQuery::builder()
        .pool(context.pool())
        .listing_type(Some(listing_type))
        .sort(sort)
        .saved_only(saved_only)
        .board_id(board_id)
        .post_id(post_id)
        .parent_id(parent_id)
        .person_id(Some(local_user.person_id))
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't get comments"))?;

      let comments = resp.comments;
  
      Ok(GetCommentsResponse { comments })
    }
  }
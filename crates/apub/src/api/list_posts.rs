use crate::{
    api::{listing_type_with_default, PerformApub},
    fetcher::resolve_actor_identifier,
    objects::board::ApubBoard,
  };
  use tinyboards_federation::config::Data;
  use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{GetPosts, GetPostsResponse},
    utils::{check_private_instance, require_user_opt},
  };
  use tinyboards_db::models::{board::boards::Board, site::local_site::LocalSite};
  use tinyboards_db_views::post_view::PostQuery;
  use tinyboards_utils::error::TinyBoardsError;

  #[async_trait::async_trait]
  impl PerformApub for GetPosts {
    type Response = GetPostsResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<GetPostsResponse, TinyBoardsError> {
        let data: &GetPosts = self;
        let local_user_view = require_user_opt(context.pool(), context.master_key(), auth).await?;
        let local_site = LocalSite::read(context.pool()).await?;

        check_private_instance(&local_user_view.clone().map(|u| u.local_user), context.pool()).await?;

        let sort = data.sort;
        let page = data.page;
        let limit = data.limit;

        let board_id = if let Some(name) = &data.board_name {
            resolve_actor_identifier::<ApubBoard, Board>(name, context, &None, true)
                .await
                .ok()
                .map(|b| b.id)
        } else {
            data.board_id
        };

        let saved_only = data.saved_only;
        let listing_type = listing_type_with_default(data.type_, &local_site, board_id)?;
        
        let resp = PostQuery::builder()
            .pool(context.pool())
            .user(local_user_view.clone().map(|l| l.local_user).as_ref())
            .listing_type(Some(listing_type))
            .sort(sort)
            .board_id(board_id)
            .saved_only(saved_only)
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't get posts"))?;

        let posts = resp.posts;

        Ok( GetPostsResponse { posts })
    }
  }
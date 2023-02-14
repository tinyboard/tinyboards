use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{ListPosts, ListPostsResponse},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{map_to_listing_type, map_to_sort_type};
use tinyboards_db_views::{post_view::PostQuery, DeleteableOrRemoveable};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListPosts {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<ListPostsResponse, TinyBoardsError> {
        let data: ListPosts = self;

        // check to see if user is logged in or not
        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check to see if the instance is private or not before listing
        check_private_instance(&user, context.pool()).await?;

        let sort = map_to_sort_type(data.sort.as_deref());
        let listing_type = map_to_listing_type(data.listing_type.as_deref());
        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let saved_only = data.saved_only;
        let user_match = user.clone();

        let mut post_query = PostQuery::builder()
            .pool(context.pool())
            .listing_type(Some(listing_type))
            .sort(Some(sort))
            .board_id(board_id)
            .user(user_match.as_ref())
            .saved_only(saved_only)
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await?;

        
        for pv in post_query
            .posts
            .iter_mut()
            .filter(|p| p.post.is_removed || p.post.is_deleted)
        {
            pv.hide_if_removed_or_deleted(user.as_ref());
        }
        

        let posts = post_query.posts;
        let total_count = post_query.count;

        Ok(ListPostsResponse { posts, total_count })
    }
}

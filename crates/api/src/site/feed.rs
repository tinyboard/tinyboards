use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::ListPostsResponse,
    site::GetFeed,
    utils::{blocking, check_private_instance, get_user_view_from_jwt_opt},
};
use tinyboards_db::{map_to_listing_type, map_to_sort_type, ListingType, SortType};
use tinyboards_db_views::{post_view::PostQuery, DeleteableOrRemoveable};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetFeed {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip_all)]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let params: &Self = &self;

        // get optional user view (don't need to be logged in to see posts)
        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // check if feed is visible or not
        check_private_instance(&user_view, context.pool()).await?;

        let listing_type = match params.listing_type.as_ref() {
            Some(ltype) => map_to_listing_type(Some(&ltype.to_lowercase())),
            None => ListingType::All,
        };

        let sort = match params.sort.as_ref() {
            Some(sort) => map_to_sort_type(Some(&sort.to_lowercase())),
            None => SortType::Hot,
        };

        let board_id = params.board_id;
        let creator_id = params.creator_id;
        let user_id = params.user_id;
        let saved_only = params.saved_only;
        let limit = params.limit;
        let page = params.page;

        let mut posts = blocking(context.pool(), move |conn| {
            PostQuery::builder()
                .conn(conn)
                .listing_type(Some(listing_type))
                .sort(Some(sort))
                .board_id(board_id)
                .user_id(user_id)
                .creator_id(creator_id)
                .saved_only(saved_only)
                .limit(limit)
                .page(page)
                .build()
                .list()
        })
        .await?
        .map_err(|_e| TinyBoardsError::err_500())?;

        if !user_view.is_some() {
            for pv in posts
                .iter_mut()
                .filter(|p| p.post.deleted || p.post.removed)
            {
                pv.hide_if_removed_or_deleted(user_view.as_ref());
            }
        }

        Ok(ListPostsResponse { posts })
    }
}

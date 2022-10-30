use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{ListPosts, ListPostsResponse},
    utils::{blocking, check_private_instance, get_user_view_from_jwt_opt},
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
        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // check to see if the instance is private or not before listing
        check_private_instance(&user_view, context.pool()).await?;

        let is_logged_in = user_view.is_some();

        let user_id = user_view.as_ref().map(|u| u.user.id);
        let sort = map_to_sort_type(data.sort.as_deref());
        let listing_type = map_to_listing_type(data.listing_type.as_deref());
        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let saved_only = data.saved_only;

        let mut posts = blocking(context.pool(), move |conn| {
            PostQuery::builder()
                .conn(conn)
                .listing_type(Some(listing_type))
                .sort(Some(sort))
                .board_id(board_id)
                .user_id(user_id)
                .saved_only(saved_only)
                .page(page)
                .limit(limit)
                .build()
                .list()
        })
        .await?
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            TinyBoardsError::err_500()
        })?;

        if !is_logged_in {
            for pv in posts
                .iter_mut()
                .filter(|p| p.post.deleted || p.post.removed)
            {
                pv.hide_if_removed_or_deleted(user_view.as_ref());
            }

            /*for pv in posts
                .iter_mut()
                .filter(|p| p.board.deleted)
            {
                pv.board = pv.to_owned().board.blank_out_deleted_info();
            }*/
        }

        Ok(ListPostsResponse { posts })
    }
}

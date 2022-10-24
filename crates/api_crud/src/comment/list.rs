use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    comment::{ListComments, ListCommentsResponse},
    utils::{blocking, get_user_view_from_jwt},
};
use porpl_db::{map_to_listing_type, map_to_comment_sort_type};
use porpl_db_views::comment_view::CommentQuery;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl <'des> PerformCrud<'des> for ListComments {
    type Response = ListCommentsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<ListCommentsResponse, PorplError> {
        let data: ListComments = self;

        let _user_view 
            = get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;
        
        let sort = map_to_comment_sort_type(data.sort.as_deref());
        let listing_type = map_to_listing_type(data.listing_type.as_deref());
        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let post_id = data.post_id;
        let parent_id = data.parent_id;
        let creator_id = data.creator_id;
        let search_term = data.search_term;
        let saved_only = data.saved_only;
        let show_deleted_and_removed = data.show_deleted_and_removed;

        let comments = blocking(context.pool(), move |conn| {
            CommentQuery::builder()
                .conn(conn)
                .listing_type(Some(listing_type))
                .sort(Some(sort))
                .board_id(board_id)
                .post_id(post_id)
                .parent_id(parent_id)
                .creator_id(creator_id)
                .search_term(search_term)
                .saved_only(saved_only)
                .show_deleted_and_removed(show_deleted_and_removed)
                .page(page)
                .limit(limit)
                .build()
                .list()
        })
        .await?
        .map_err(|_| PorplError::err_500())?;

        Ok(ListCommentsResponse { comments })
    }
}
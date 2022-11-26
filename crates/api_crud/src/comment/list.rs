use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{ListComments, ListCommentsResponse},
    data::TinyBoardsContext,
    post::{GetPostComments, PostIdPath},
    utils::{blocking, check_private_instance, get_user_view_from_jwt_opt},
};
use tinyboards_db::{map_to_comment_sort_type, map_to_listing_type, models::post::post::Post};
use tinyboards_db_views::{
    comment_view::CommentQuery, structs::CommentView, DeleteableOrRemoveable,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListComments {
    type Response = ListCommentsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<ListCommentsResponse, TinyBoardsError> {
        let data: ListComments = self;

        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // check if instance is private before listing comments
        check_private_instance(&user_view, context.pool()).await?;

        let user_id = match user_view {
            Some(ref user_view) => Some(user_view.user.id),
            None => None,
        };
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

        let mut comments = blocking(context.pool(), move |conn| {
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
                .user_id(user_id)
                .page(page)
                .limit(limit)
                .build()
                .list()
        })
        .await??;

        // blank out comment info if deleted or removed
        for cv in comments.iter_mut()
        //.filter(|cv| cv.comment.deleted || cv.comment.removed)
        {
            cv.hide_if_removed_or_deleted(user_view.as_ref());
        }

        // order into tree
        let comments = CommentView::into_tree(comments);

        Ok(ListCommentsResponse { comments: comments })
    }
}

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPostComments {
    type Response = Vec<CommentView>;
    type Route = PostIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // check if instance is private before listing comments
        check_private_instance(&user_view, context.pool()).await?;

        // check if post exists
        if blocking(context.pool(), move |conn| {
            Post::check_if_exists(conn, path.post_id)
        })
        .await??
        .is_none()
        {
            return Err(TinyBoardsError::from_message("invalid post id"));
        }

        let mut comments = blocking(context.pool(), move |conn| {
            CommentQuery::builder()
                .conn(conn)
                //.sort(None)
                .post_id(Some(path.post_id))
                .show_deleted_and_removed(Some(true))
                //.page(None)
                //.limit(None)
                .build()
                .list()
        })
        .await??;

        // blank out comment info if deleted or removed
        for cv in comments.iter_mut()
        //.filter(|cv| cv.comment.deleted || cv.comment.removed)
        {
            cv.hide_if_removed_or_deleted(user_view.as_ref());
        }

        let comments = CommentView::into_tree(comments);

        Ok(comments)
    }
}

use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{ListComments, ListCommentsResponse},
    data::TinyBoardsContext,
    post::{GetPostComments, PostIdPath},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{
    map_to_comment_sort_type, map_to_listing_type, models::post::posts::Post, CommentSortType,
    ListingType,
};
use tinyboards_db_views::{
    comment_view::CommentQuery, structs::CommentView, DeleteableOrRemoveable,
};
use tinyboards_utils::error::TinyBoardsError;

#[derive(PartialEq)]
enum Format {
    List,
    Tree,
}

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

        let local_user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if instance is private before listing comments
        check_private_instance(&local_user, context.pool()).await?;

        let person_id = match local_user {
            Some(ref local_user) => Some(local_user.person.id),
            None => None,
        };

        let sort = match data.sort.as_ref() {
            Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
            None => CommentSortType::Hot,
        };

        let listing_type = match data.listing_type.as_ref() {
            Some(listing_type) => map_to_listing_type(Some(&listing_type.to_lowercase())),
            None => ListingType::All,
        };

        let format = match data.format {
            Some(format) => match format.to_lowercase().as_ref() {
                "list" => Format::List,
                _ => Format::Tree,
            },
            None => Format::Tree,
        };

        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let post_id = data.post_id;
        let parent_id = data.parent_id;
        let creator_id = data.creator_id;
        let search_term = data.search_term;
        let saved_only = data.saved_only;
        let show_deleted_and_removed = format == Format::Tree || data.show_deleted_and_removed.unwrap_or(true);

        let response = CommentQuery::builder()
            .pool(context.pool())
            .listing_type(Some(listing_type))
            .sort(Some(sort))
            .board_id(board_id)
            .post_id(post_id)
            .parent_id(parent_id)
            .creator_id(creator_id)
            .search_term(search_term)
            .saved_only(saved_only)
            .show_deleted(Some(show_deleted_and_removed))
            .show_removed(Some(show_deleted_and_removed))
            .person_id(person_id)
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await?;

        let mut comments = response.comments;

        let total_count = response.count;

        // blank out comment info if deleted or removed
        for cv in comments
            .iter_mut()
            .filter(|cv| cv.comment.is_deleted || cv.comment.is_removed)
        {
            cv.hide_if_removed_or_deleted(local_user.as_ref());
        }

        if let Format::Tree = format {
            // order into tree
            comments = CommentView::into_tree(comments, parent_id);
        }

        Ok(ListCommentsResponse {
            comments,
            total_count,
        })
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
        let local_user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if instance is private before listing comments
        check_private_instance(&local_user, context.pool()).await?;

        // check if post exists
        if Post::check_if_exists(context.pool(), path.post_id)
            .await?
            .is_none()
        {
            return Err(TinyBoardsError::from_message(400, "invalid post id"));
        }

        let response = CommentQuery::builder()
            .pool(context.pool())
            //.sort(None)
            .post_id(Some(path.post_id))
            .show_deleted(Some(true))
            .show_removed(Some(true))
            //.page(None)
            //.limit(None)
            .build()
            .list()
            .await?;

        let mut comments = response.comments;

        // blank out comment info if deleted or removed
        for cv in comments
            .iter_mut()
            .filter(|cv| cv.comment.is_deleted || cv.comment.is_removed)
        {
            cv.hide_if_removed_or_deleted(local_user.as_ref());
        }

        let comments = CommentView::into_tree(comments, None);

        Ok(comments)
    }
}

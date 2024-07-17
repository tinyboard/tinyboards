use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api::local_user;
use tinyboards_api_common::{
    comment::{ListComments, ListCommentsResponse},
    data::TinyBoardsContext,
    post::{GetPostComments, PostIdPath},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{
    map_to_comment_sort_type, map_to_listing_type,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        person::local_user::AdminPerms,
        post::posts::Post,
    },
    traits::Crud,
    CommentSortType, ListingType,
};
use tinyboards_db_views::{
    comment_view::CommentQuery,
    structs::{CommentView, LocalUserView},
    DeleteableOrRemoveable,
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

        let person_id_ = match local_user {
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
        let show_deleted_and_removed =
            format == Format::Tree || data.show_deleted_and_removed.unwrap_or(true);

        let is_admin = match local_user {
            Some(ref local_user_view) => local_user_view
                .local_user
                .has_permission(AdminPerms::Content),
            None => false,
        };

        // only check mod status if board id is provided
        let is_mod = match person_id_ {
            Some(person_id_) => match board_id {
                Some(board_id) => {
                    let board_mod = BoardModerator::get_by_person_id_for_board(
                        context.pool(),
                        person_id_,
                        board_id,
                        true,
                    )
                    .await;

                    match board_mod {
                        Ok(m) => m.has_permission(ModPerms::Content),
                        Err(_) => false,
                    }
                }
                None => false,
            },

            None => false,
        };

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
            .person_id(person_id_)
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
            cv.hide_if_removed_or_deleted(
                local_user.as_ref().map(|view| view.person.id),
                is_admin,
                is_mod,
            );
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
        PostIdPath { post_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let local_user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if instance is private before listing comments
        check_private_instance(&local_user, context.pool()).await?;

        // check if post exists
        /*if Post::check_if_exists(context.pool(), path.post_id)
            .await?
            .is_none()
        {
            return Err(TinyBoardsError::from_message(400, "invalid post id"));
            }*/

        let post = Post::read(context.pool(), post_id).await.map_err(|e| {
            TinyBoardsError::from_error_message(
                e,
                500,
                "Failed to load post. Provided post id may be invalid.",
            )
        })?;

        let board_id = post.board_id;
        let is_admin = match local_user {
            Some(LocalUserView { ref local_user, .. }) => {
                local_user.has_permission(AdminPerms::Content)
            }
            None => false,
        };

        let is_mod = match local_user {
            Some(LocalUserView { ref person, .. }) => {
                let mod_rel = BoardModerator::get_by_person_id_for_board(
                    context.pool(),
                    person.id,
                    board_id,
                    true,
                )
                .await;

                match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                }
            }
            None => false,
        };

        let response = CommentQuery::builder()
            .pool(context.pool())
            //.sort(None)
            .post_id(Some(post_id))
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
            cv.hide_if_removed_or_deleted(
                local_user.as_ref().map(|view| view.person.id),
                is_admin,
                is_mod,
            );
        }

        let comments = CommentView::into_tree(comments, None);

        Ok(comments)
    }
}

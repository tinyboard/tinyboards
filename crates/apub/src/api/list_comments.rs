use crate::{
    api::{listing_type_with_default, PerformApub},
    fetcher::resolve_actor_identifier,
    objects::board::ApubBoard,
};
use tinyboards_api_common::{
    comment::{GetComments, GetCommentsResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{
    map_to_comment_sort_type,
    models::{board::boards::Board, person::local_user::AdminPerms, site::local_site::LocalSite},
};
use tinyboards_db_views::comment_view::CommentQuery;
use tinyboards_db_views::{structs::CommentView, DeleteableOrRemoveable};
use tinyboards_federation::config::Data;
use tinyboards_utils::error::TinyBoardsError;

#[derive(PartialEq)]
enum Format {
    List,
    Tree,
}

#[async_trait::async_trait]
impl PerformApub for GetComments {
    type Response = GetCommentsResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        &self,
        context: &Data<TinyBoardsContext>,
        auth: Option<&str>,
    ) -> Result<GetCommentsResponse, TinyBoardsError> {
        let data: &GetComments = self;
        let local_user_view = load_user_opt(context.pool(), context.master_key(), auth).await?;
        let local_site = LocalSite::read(context.pool()).await?;
        check_private_instance(&local_user_view, context.pool()).await?;

        let board_id = if let Some(name) = &data.board_name {
            resolve_actor_identifier::<ApubBoard, Board>(name, context, &None, true)
                .await
                .ok()
                .map(|b| b.id)
        } else {
            data.board_id
        };
        let sort = data.sort.clone().map(|x| x.to_lowercase());
        let sort = Some(map_to_comment_sort_type(sort.as_deref()));
        let saved_only = data.saved_only;
        let page = data.page;
        let limit = data.limit;
        let creator_id = data.creator_id;
        let parent_id = data.parent_id;
        let listing_type = listing_type_with_default(data.type_, &local_site, board_id)?;
        let post_id = data.post_id;

        let format = match data.format {
            Some(ref format) => match format.to_lowercase().as_ref() {
                "list" => Format::List,
                _ => Format::Tree,
            },
            None => Format::Tree,
        };

        let is_admin = match &local_user_view {
            Some(v) => v.local_user.has_permission(AdminPerms::Content),
            None => false,
        };

        let own_comments = match creator_id {
            Some(creator_id) => match &local_user_view {
                Some(v) => v.person.id == creator_id,
                None => false,
            },
            None => false,
        };

        let show_deleted_and_removed = format == Format::Tree || own_comments || is_admin;

        let resp = CommentQuery::builder()
            .pool(context.pool())
            .listing_type(Some(listing_type))
            .sort(sort)
            .show_deleted(Some(show_deleted_and_removed))
            .show_removed(Some(show_deleted_and_removed))
            .saved_only(saved_only)
            .board_id(board_id)
            .post_id(post_id)
            .creator_id(creator_id)
            .parent_id(parent_id)
            .person_id(local_user_view.clone().map(|u| u.local_user.person_id))
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't get comments"))?;

        let mut comments = resp.comments;
        let total_count = resp.count;

        // let local_user = local_user_view.map(|u| u.local_user);

        // blank out comment info if deleted or removed
        for cv in comments
            .iter_mut()
            .filter(|cv| cv.comment.is_deleted || cv.comment.is_removed)
        {
            cv.hide_if_removed_or_deleted(local_user_view.as_ref());
        }

        if let Format::Tree = format {
            comments = CommentView::into_tree(comments, parent_id);
        }

        Ok(GetCommentsResponse {
            comments,
            total_count,
        })
    }
}

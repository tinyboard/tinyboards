use crate::PerformCrud;
use actix_web::web;
use tinyboards_api_common::{
    comment::{CommentIdPath, GetComment, ListCommentsResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{
    map_to_comment_sort_type,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        comment::comments::Comment,
        person::local_user::AdminPerms,
    },
    traits::Crud,
    CommentSortType,
};
use tinyboards_db_views::{
    comment_view::CommentQueryResponse,
    structs::{CommentView, LocalUserView},
};
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetComment {
    type Response = ListCommentsResponse;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &web::Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data = self;

        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if the instance is private before listing comments
        check_private_instance(&v, context.pool()).await?;

        let person = v.as_ref();
        let comment_id = path.comment_id;
        let post_id = data.post;

        let c = Comment::read(context.pool(), comment_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read comment."))?;

        let board_id = c.board_id;

        let is_admin = match v {
            Some(LocalUserView { ref local_user, .. }) => {
                local_user.has_permission(AdminPerms::Content)
            }
            None => false,
        };

        let is_mod = match v {
            Some(LocalUserView { ref person, .. }) => {
                let board_mod = BoardModerator::get_by_person_id_for_board(
                    context.pool(),
                    person.id,
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
        };

        let comment_context = data.context;
        let sort = match data.sort.as_ref() {
            Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
            None => CommentSortType::Hot,
        };

        let CommentQueryResponse { comments, count } = CommentView::get_comment_with_replies(
            context.pool(),
            comment_id,
            Some(sort),
            person,
            comment_context,
            post_id,
            is_admin,
            is_mod,
        )
        .await?;

        Ok(ListCommentsResponse {
            comments,
            total_count: count,
        })
    }
}

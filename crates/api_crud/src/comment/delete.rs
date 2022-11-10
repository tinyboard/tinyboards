use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CommentResponse, DeleteComment},
    data::TinyBoardsContext,
    utils::{
        blocking, check_board_ban, check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked, check_user_valid, get_user_view_from_jwt,
    },
};
use tinyboards_db::{
    models::{comment::comment::Comment, post::post::Post},
    traits::Crud,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &DeleteComment = &self;

        let user_view = get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let comment_id = path.comment_id;

        let orig_comment = blocking(context.pool(), move |conn| {
            Comment::read(conn, comment_id)
                .map_err(|_e| TinyBoardsError::from_string("original comment not found", 404))
        })
        .await??;

        let orig_post = blocking(context.pool(), move |conn| {
            Post::read(conn, orig_comment.post_id)
                .map_err(|_e| TinyBoardsError::from_string("original post not found", 404))
        })
        .await??;

        if orig_comment.deleted == data.deleted {
            return Err(TinyBoardsError::from_string(
                "couldn't delete comment again",
                500,
            ));
        }

        check_board_ban(user_view.user.id, orig_post.board_id, context.pool()).await?;

        check_board_deleted_or_removed(orig_post.board_id, context.pool()).await?;

        check_post_deleted_removed_or_locked(orig_post.id, context.pool()).await?;

        check_user_valid(
            user_view.user.banned,
            user_view.user.expires,
            user_view.user.deleted,
        )?;

        if !Comment::is_comment_creator(user_view.user.id, orig_comment.creator_id) {
            return Err(TinyBoardsError::from_string(
                "comment edit not allowed",
                405,
            ));
        }

        let deleted = data.deleted;

        blocking(context.pool(), move |conn| {
            Comment::update_deleted(conn, comment_id, deleted)
                .map_err(|_e| TinyBoardsError::err_500())
        })
        .await??;

        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(user_view.user.id))
                .map_err(|_e| TinyBoardsError::from_string("could not find comment", 404))
        })
        .await??;

        Ok(CommentResponse { comment_view })
    }
}

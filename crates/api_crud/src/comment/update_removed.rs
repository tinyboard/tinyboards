use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    build_response::build_comment_response,
    comment::{CommentIdPath, CommentResponse, ToggleCommentRemove},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        board::boards::Board,
        comment::comments::Comment,
        moderator::mod_actions::{ModRemoveComment, ModRemoveCommentForm},
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ToggleCommentRemove {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        CommentIdPath { comment_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ToggleCommentRemove = &self;
        let comment = Comment::read(context.pool(), comment_id).await?;
        let orig_board = Board::read(context.pool(), comment.board_id).await?;

        // only board mod allowed
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), orig_board.id, ModPerms::Content)
            .await
            .unwrap()?;

        let removed = data.value;
        comment
            .set_removed(context.pool(), removed)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to set comment removed status.")
            })?;

        Comment::resolve_reports(context.pool(), comment.id, view.person.id).await?;

        // mod log
        let form = ModRemoveCommentForm {
            mod_person_id: view.person.id,
            comment_id: comment.id,
            removed: Some(Some(removed)),
            reason: Some(data.reason.clone()),
        };

        ModRemoveComment::create(context.pool(), &form).await?;

        /*let post_id = updated_comment.post_id;
        let post = Post::read(context.pool(), post_id).await?;
        let recipient_ids = send_local_notifs(
            vec![],
            &updated_comment,
            &view.person.clone(),
            &post,
            false,
            context,
        )
        .await?;*/

        Ok(build_comment_response(context, comment.id, Some(view), vec![]).await?)
    }
}

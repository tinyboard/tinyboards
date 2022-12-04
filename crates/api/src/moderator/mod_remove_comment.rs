use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{RemoveComment, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModRemoveComment, ModRemoveCommentForm},
    models::comment::comment::Comment,
    traits::Crud,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl <'des> Perform<'des> for RemoveComment {
    type Response = ModActionResponse<ModRemoveComment>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &RemoveComment = &self;

        let comment_id = data.comment_id;
        let reason = data.reason.clone();
        let removed = data.removed;

        // get the comment object
        let orig_comment = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id.clone(), None)
        })
        .await??;

        // require a mod/admin for this action
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(orig_comment.board.id, context.pool())
            .await
            .unwrap()?;

        // update the comment in the database
        blocking(context.pool(), move |conn| {
            Comment::update_removed(conn, comment_id, removed)
        })
        .await??;

        // form for submitting remove action to mod log
        let remove_comment_form = ModRemoveCommentForm {
            mod_user_id: user.id,
            comment_id: comment_id.clone(),
            reason: Some(reason),
            removed: Some(Some(removed.clone()))
        };

        // submit mod action to the mod log
        let mod_action = blocking(context.pool(), move |conn| {
            ModRemoveComment::create(conn, &remove_comment_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}
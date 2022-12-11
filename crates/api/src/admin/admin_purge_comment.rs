use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::PurgeComment,
    data::TinyBoardsContext,
    moderator::ModActionResponse,
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::{
        comment::comments::Comment,
        moderator::admin_actions::{AdminPurgeComment, AdminPurgeCommentForm},
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgeComment {
    type Response = ModActionResponse<AdminPurgeComment>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgeComment = &self;

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let target_comment_id = data.comment_id;
        let reason = data.reason.clone();

        // delete comment
        blocking(context.pool(), move |conn| {
            Comment::delete(conn, target_comment_id)
        })
        .await??;

        let form = AdminPurgeCommentForm {
            admin_id: user.id,
            comment_id: target_comment_id,
            reason: Some(reason),
        };

        // submit mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            AdminPurgeComment::create(conn, &form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}

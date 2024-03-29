use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, DeleteComment, CommentResponse},
    data::TinyBoardsContext,
    utils::require_user, build_response::build_comment_response,
};
use tinyboards_db::{
    models::comment::comments::Comment,
    traits::Crud,
};
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

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let comment_id = path.comment_id;

        let orig_comment = Comment::read(context.pool(), comment_id).await?;

        if orig_comment.is_deleted == data.deleted {
            return Err(TinyBoardsError::from_message(
                400,
                "couldn't delete comment a second time!",
            ));
        }

        if !Comment::is_comment_creator(view.person.id, orig_comment.creator_id) {
            return Err(TinyBoardsError::from_message(
                403,
                "comment edit not allowed",
            ));
        }

        let deleted = data.deleted;

        let deleted_comment = Comment::update_deleted(context.pool(), comment_id, deleted).await?;

        Ok(build_comment_response(context, deleted_comment.id, Some(view), vec![]).await?)
    }
}

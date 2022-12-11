use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, DeleteComment},
    data::TinyBoardsContext,
    site::Message,
    utils::{
        blocking, 
        require_user,
    },
};
use tinyboards_db::{
    models::{comment::comment::Comment, post::post::Post},
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteComment {
    type Response = Message;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &DeleteComment = &self;

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let comment_id = path.comment_id;

        let orig_comment = blocking(context.pool(), move |conn| {
            Comment::read(conn, comment_id)
        })
        .await??;

        let orig_post = blocking(context.pool(), move |conn| {
            Post::read(conn, orig_comment.post_id)
        })
        .await??;

        if orig_comment.deleted == data.deleted {
            return Err(TinyBoardsError::from_message("couldn't delete comment a second time!"));
        }

        if !Comment::is_comment_creator(user.id, orig_comment.creator_id) {
            return Err(TinyBoardsError::from_message("comment edit not allowed"));
        }

        let deleted = data.deleted;

        blocking(context.pool(), move |conn| {
            Comment::update_deleted(conn, comment_id, deleted)
        })
        .await??;

        Ok(Message::new("Comment deleted!"))
    }
}

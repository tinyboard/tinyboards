use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{DeletePost, PostIdPath},
    site::Message,
    utils::{blocking, require_user},
};
use tinyboards_db::{models::post::posts::Post, traits::Crud};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeletePost {
    type Response = Message;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &DeletePost = &self;
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let post_id = path.post_id;
        let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

        if orig_post.is_deleted == data.deleted {
            return Err(TinyBoardsError::from_message(
                400,
                "couldn't delete post a second time!",
            ));
        }

        if !Post::is_post_creator(user.id, orig_post.creator_id) {
            return Err(TinyBoardsError::from_message(403, "post edit not allowed"));
        }

        let post_id = path.post_id;
        let deleted = data.deleted;

        blocking(context.pool(), move |conn| {
            Post::update_deleted(conn, post_id, deleted)
        })
        .await??;

        Ok(Message::new("Post deleted!"))
    }
}

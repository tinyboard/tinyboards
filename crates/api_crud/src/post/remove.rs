use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    post::{RemovePost, PostResponse},
    data::TinyBoardsContext,
    utils::{require_user}, build_response::{build_post_response},
};
use tinyboards_db::{
    models::{moderator::mod_actions::{ModRemovePostForm, ModRemovePost}, post::posts::Post},
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for RemovePost {
    type Response = PostResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &RemovePost = &self;
        let orig_post = Post::read(context.pool(), data.post_id).await?;

        // require board mod
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(orig_post.board_id, context.pool())
            .await
            .unwrap()?;

        let post_id = orig_post.id;
        let removed = data.removed;
        let updated_post = Post::update_removed(context.pool(), post_id, removed).await?;

        // mod log
        let form = ModRemovePostForm {
            mod_person_id: view.person.id,
            post_id: updated_post.id,
            removed: Some(Some(removed)),
            reason: Some(data.reason.clone()),

        };
        ModRemovePost::create(context.pool(), &form).await?;

        Ok(build_post_response(context, updated_post.board_id, view.person.id, post_id).await?)
    }
}
use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{ModActionResponse, RemovePost},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModRemovePost, ModRemovePostForm},
    models::post::posts::Post,
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for RemovePost {
    type Response = ModActionResponse<ModRemovePost>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &RemovePost = &self;

        let post_id = data.post_id;
        let reason = data.reason.clone();
        let removed = data.removed;

        // get the post object
        let orig_post = blocking(context.pool(), move |conn| {
            Post::read(conn, post_id.clone())
        })
        .await??;

        // require a mod/admin for this action
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(orig_post.board_id, context.pool())
            .await
            .unwrap()?;

        // update post in the database
        blocking(context.pool(), move |conn| {
            Post::update_removed(conn, post_id, removed)
        })
        .await??;

        // form for submitting remove action to mod log
        let remove_post_form = ModRemovePostForm {
            mod_user_id: user.id,
            post_id: post_id.clone(),
            reason: Some(reason),
            removed: Some(Some(removed.clone())),
        };

        // submit mod action to mod log
        let mod_action = blocking(context.pool(), move |conn| {
            ModRemovePost::create(conn, &remove_post_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}

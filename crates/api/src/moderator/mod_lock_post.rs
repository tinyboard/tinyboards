use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{LockPost, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModLockPost, ModLockPostForm},
    models::post::post::Post,
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for LockPost {
    type Response = ModActionResponse<ModLockPost>;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &LockPost = &self;

        let post_id = data.post_id;
        let locked = data.locked;

        // get the post object
        let orig_post = blocking(context.pool(), move |conn| {
            Post::read(conn, post_id.clone())
                .map_err(|_e| TinyBoardsError::from_string("couldn't find post", 404))
        })
        .await??;

        // require a mod/admin user for this
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(orig_post.board_id, context.pool())
            .await
            .unwrap()?;

        // update the post in the database to be locked
        blocking(context.pool(), move |conn| {
            Post::update_locked(conn, post_id.clone(), locked.clone())
                .map_err(|_e| TinyBoardsError::from_string("could not lock post", 500))
        })
        .await??;

        // form for submitting lock action for mod log
        let lock_form = ModLockPostForm {
            mod_user_id: user.id,
            post_id: post_id.clone(),
            locked: Some(Some(locked.clone())),
        };

        // enter mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            ModLockPost::create(conn, &lock_form)
                .map_err(|_e| TinyBoardsError::from_string("could not log mod action", 500))
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}

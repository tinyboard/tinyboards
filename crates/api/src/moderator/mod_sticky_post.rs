use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{ModActionResponse, StickyPost},
    utils::require_user,
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModStickyPost, ModStickyPostForm},
    models::post::posts::Post,
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for StickyPost {
    type Response = ModActionResponse<ModStickyPost>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &StickyPost = &self;

        let post_id = data.post_id;
        let stickied = data.stickied;

        // get the post object
        let orig_post = Post::read(context.pool(), post_id.clone()).await?;

        // require a mod/admin for this action
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(orig_post.board_id, context.pool())
            .await
            .unwrap()?;

        // update the post in the database to be stickied (or un-stickied)
        Post::update_stickied(context.pool(), post_id.clone(), stickied.clone()).await?;

        // form for submitting post sticky action to the mod log
        let sticky_post_form = ModStickyPostForm {
            mod_person_id: view.person.id,
            post_id: post_id.clone(),
            stickied: Some(Some(stickied.clone())),
        };

        // submit mod action to the mod log
        let mod_action = ModStickyPost::create(context.pool(), &sticky_post_form).await?;

        Ok(ModActionResponse { mod_action })
    }
}

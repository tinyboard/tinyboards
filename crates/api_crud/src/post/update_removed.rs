use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    build_response::build_post_response,
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, TogglePostRemove},
    utils::require_user,
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        moderator::mod_actions::{ModRemovePost, ModRemovePostForm},
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for TogglePostRemove {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        PostIdPath { post_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &TogglePostRemove = &self;
        let orig_post = Post::read(context.pool(), post_id).await?;

        // require board mod
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), orig_post.board_id, ModPerms::Content, None)
            .await
            .unwrap()?;

        let post_id = orig_post.id;
        let removed = data.value;
        let updated_post = Post::update_removed(context.pool(), post_id, removed).await?;

        Post::resolve_reports(context.pool(), post_id, view.person.id).await?;

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

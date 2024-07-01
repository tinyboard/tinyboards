use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    build_response::build_post_response,
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, TogglePostLock},
    utils::{check_board_ban, check_board_deleted_or_removed, require_user},
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        board::boards::Board,
        moderator::mod_actions::{ModLockPost, ModLockPostForm},
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for TogglePostLock {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        PostIdPath { post_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<PostResponse, TinyBoardsError> {
        let data: &TogglePostLock = &self;
        let post = Post::read(context.pool(), post_id).await?;
        let board = Board::read(context.pool(), post.board_id).await?;

        // require board mod (minimum)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), board.id, ModPerms::Content)
            .await
            .unwrap()?;

        // validations
        check_board_ban(view.person.id, board.id, context.pool()).await?;

        let locked = data.value;

        // lock or unlock post
        post.set_locked(context.pool(), locked).await?;

        // mod log
        let mod_lock_post_form = ModLockPostForm {
            mod_person_id: view.person.id,
            post_id: post_id,
            locked: Some(Some(locked)),
        };
        ModLockPost::create(context.pool(), &mod_lock_post_form).await?;

        Ok(build_post_response(context, board.id, view.person.id, post_id).await?)
    }
}

use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, SavePost},
    utils::{require_user, is_mod_or_admin},
};
use tinyboards_db::{
    models::post::{post_saved::{PostSaved, PostSavedForm}, posts::Post},
    traits::{Crud, Saveable},
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SavePost {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &SavePost = &self;

        let local_user_view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;

        let saved_form = PostSavedForm {
            post_id: path.post_id,
            person_id: local_user_view.person.id,
        };

        if data.save {
            PostSaved::save(context.pool(), &saved_form).await?;
        } else {
            PostSaved::unsave(context.pool(), &saved_form).await?;
        }

        let post_id = path.post_id;
        let board_id = Post::read(context.pool(), post_id).await?.board_id;
        let person_id = local_user_view.person.id;

        let is_mod_or_admin = is_mod_or_admin(context.pool(), local_user_view.person.id, board_id).await.is_ok();

        let post_view = PostView::read(context.pool(), post_id, Some(person_id), Some(is_mod_or_admin)).await?;

        Ok(PostResponse { post_view })
    }
}

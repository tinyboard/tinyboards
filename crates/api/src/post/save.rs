use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, SavePost},
    utils::{get_user_view_from_jwt},
};
use tinyboards_db::{
    models::post::user_post_save::{PostSaved, PostSavedForm},
    traits::Saveable,
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

        let user_view = get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let saved_form = PostSavedForm {
            post_id: path.post_id,
            user_id: user_view.user.id,
        };

        if data.save {
            PostSaved::save(context.pool(), &saved_form).await?;
        } else {
            PostSaved::unsave(context.pool(), &saved_form).await?;
        }

        let post_id = path.post_id;
        let user_id = user_view.user.id;
        let post_view = PostView::read(context.pool(), post_id, Some(user_id)).await?;

        Ok(PostResponse { post_view })
    }
}

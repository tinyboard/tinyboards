use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, SavePost},
    utils::{get_local_user_view_from_jwt},
};
use tinyboards_db::{
    models::post::post_saved::{PostSaved, PostSavedForm},
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

        let local_user_view = get_local_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

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
        let person_id = local_user_view.person.id;
        let post_view = PostView::read(context.pool(), post_id, Some(person_id)).await?;

        Ok(PostResponse { post_view })
    }
}

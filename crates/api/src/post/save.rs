use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    post::{SavePost, PostResponse},
    utils::{
        blocking,
        get_user_view_from_jwt,
    },
    data::PorplContext,
};
use porpl_db::{
    models::post::post_saved::{PostSaved, PostSavedForm},
    traits::Saveable,
};
use porpl_db_views::structs::PostView;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SavePost {
    type Response = PostResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data: &SavePost = &self;

        let user_view =
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;
        
        let saved_form = PostSavedForm {
            post_id: data.post_id,
            user_id: user_view.user.id,
        };

        if data.save {
            let save_post = move |conn: &mut _| PostSaved::save(conn, &saved_form);
            blocking(context.pool(), save_post)
                .await?
                .map_err(|_e| PorplError::from_string("could not save post", 500))?;
        } else {
            let unsave_post = move |conn: &mut _| PostSaved::unsave(conn, &saved_form);
            blocking(context.pool(), unsave_post)
                .await?
                .map_err(|_e| PorplError::from_string("could not unsave post", 500))?;
        }

        let post_id = data.post_id;
        let user_id = user_view.user.id;
        let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, post_id, Some(user_id))
                .map_err(|_e| PorplError::from_string("could not find post", 404))
        })
        .await??;

        Ok( PostResponse { post_view } )

    }
}
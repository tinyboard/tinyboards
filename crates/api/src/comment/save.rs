use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    comment::{SaveComment, CommentResponse, CommentIdPath},
    utils::{
        blocking,
        get_user_view_from_jwt,
    }, 
    data::PorplContext,
};
use porpl_db::{
    models::comment::comment_saved::{CommentSaved, CommentSavedForm},
    traits::Saveable,
};
use porpl_db_views::structs::CommentView;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SaveComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data: &SaveComment = &self;

        let user_view =
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;
        
        let saved_form = CommentSavedForm {
            comment_id: path.comment_id,
            user_id: user_view.user.id,
        };

        if data.save {
            let save_comment = move |conn: &mut _| CommentSaved::save(conn, &saved_form);
            blocking(context.pool(), save_comment)
                .await?
                .map_err(|_e| PorplError::from_string("could not save comment", 500))?;
        } else {
            let unsave_comment = move |conn: &mut _| CommentSaved::unsave(conn, &saved_form);
            blocking(context.pool(), unsave_comment)
                .await?
                .map_err(|_e| PorplError::from_string("could not unsave comment", 500))?;
        }

        let comment_id = path.comment_id;
        let user_id = user_view.user.id;
        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(user_id))
                .map_err(|_e| PorplError::from_string("could not find comment", 404))
        })
        .await??;

        Ok( CommentResponse { comment_view } )
    }
}
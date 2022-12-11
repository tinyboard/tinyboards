use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CommentResponse, SaveComment},
    data::TinyBoardsContext,
    utils::{blocking, get_user_view_from_jwt},
};
use tinyboards_db::{
    models::comment::user_comment_save::{CommentSaved, CommentSavedForm},
    traits::Saveable,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SaveComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &SaveComment = &self;

        let user_view = get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let saved_form = CommentSavedForm {
            comment_id: path.comment_id,
            user_id: user_view.user.id,
        };

        if data.save {
            let save_comment = move |conn: &mut _| CommentSaved::save(conn, &saved_form);
            blocking(context.pool(), save_comment)
                .await?
                .map_err(|_e| TinyBoardsError::from_message("could not save comment"))?;
        } else {
            let unsave_comment = move |conn: &mut _| CommentSaved::unsave(conn, &saved_form);
            blocking(context.pool(), unsave_comment)
                .await?
                .map_err(|_e| TinyBoardsError::from_message("could not unsave comment"))?;
        }

        let comment_id = path.comment_id;
        let user_id = user_view.user.id;
        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(user_id))
                .map_err(|_e| TinyBoardsError::from_message("could not find comment"))
        })
        .await??;

        Ok(CommentResponse { comment_view })
    }
}

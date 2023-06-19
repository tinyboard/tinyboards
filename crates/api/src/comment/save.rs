use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CommentResponse, SaveComment},
    data::TinyBoardsContext,
    utils::{get_local_user_view_from_jwt}, build_response::build_comment_response,
};
use tinyboards_db::{
    models::comment::comment_saved::{CommentSaved, CommentSavedForm},
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

        let local_user_view = get_local_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let saved_form = CommentSavedForm {
            comment_id: path.comment_id,
            person_id: local_user_view.person.id,
        };

        if data.save {
            CommentSaved::save(context.pool(), &saved_form)
                .await
                .map_err(|_e| TinyBoardsError::from_message(500, "could not save comment"))?;

        } else {
            CommentSaved::unsave(context.pool(), &saved_form)
                .await
                .map_err(|_e| TinyBoardsError::from_message(500, "could not unsave comment"))?;
        }

        let comment_id = path.comment_id;
        let person_id = local_user_view.person.id;
        let comment_view = CommentView::read(context.pool(), comment_id, Some(person_id)).await?;

        Ok(build_comment_response(context, comment_id, Some(local_user_view), None, vec![]).await?)
    }
}

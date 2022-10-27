use actix_web::web::Data;
use porpl_api_common::{
    comment::{CommentResponse, EditComment, CommentIdPath},
    utils::{
        blocking,
        check_board_ban,
        check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked,
        check_comment_deleted_or_removed,
        get_user_view_from_jwt,
    },
    data::PorplContext,
};
use porpl_db::{
    models::comment::comment::{Comment, CommentForm},
    traits::Crud,
};
use porpl_db_views::structs::CommentView;
use porpl_utils::error::PorplError;
use crate::PerformCrud;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<CommentResponse, PorplError> {
        let data: &EditComment = &self;
        let user_view =
            get_user_view_from_jwt(auth.unwrap_or(""), context.pool(), context.master_key()).await?;

        let comment_id = path.comment_id;
        let orig_comment = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, None)
                .map_err(|_e| PorplError::from_string("could not find original comment", 404))
        })
        .await??;

        check_board_ban(
            user_view.user.id, 
            orig_comment.board.id, 
            context.pool(),
        )
        .await?;

        check_board_deleted_or_removed(
            orig_comment.board.id, 
            context.pool(),
        )
        .await?;

        check_post_deleted_removed_or_locked(
            orig_comment.post.id, 
            context.pool(),
        )
        .await?;

        check_comment_deleted_or_removed(
            orig_comment.comment.id, 
            context.pool(),
        )
        .await?;

        if user_view.user.id != orig_comment.creator.id  {
            return Err(PorplError::from_string("comment edit not allowed", 405));
        }

        let body = data.body.clone();
        let body_html = data.body_html.clone();
        let comment_id = path.comment_id;

        let form = CommentForm {
            creator_id: orig_comment.comment.creator_id,
            post_id: orig_comment.comment.post_id,
            body,
            body_html,
            ..CommentForm::default()
        };

        blocking(context.pool(), move |conn| {
            Comment::update(conn, comment_id, &form)
                .map_err(|_e| PorplError::from_string("could not update comment", 500))
        })
        .await??;

        // do mentions parsing here
        // do mentions notifications sending here

        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(orig_comment.creator.id))
                .map_err(|_e| PorplError::from_string("could not find updated comment", 500))
        })
        .await??;

        Ok( CommentResponse { comment_view } )
    }
}
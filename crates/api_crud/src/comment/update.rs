use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CommentResponse, EditComment},
    data::TinyBoardsContext,
    utils::{
        blocking, check_board_deleted_or_removed, check_comment_deleted_or_removed,
        check_post_deleted_or_removed, require_user,
    },
};
use tinyboards_db::{
    models::comment::comments::{Comment, CommentForm},
    traits::Crud,
    utils::naive_now,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::{error::TinyBoardsError, parser::parse_markdown};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<CommentResponse, TinyBoardsError> {
        let data: &EditComment = &self;
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        let comment_id = path.comment_id;
        let orig_comment = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, None)
        })
        .await??;

        check_board_deleted_or_removed(orig_comment.board.id, context.pool()).await?;

        check_post_deleted_or_removed(orig_comment.post.id, context.pool()).await?;

        check_comment_deleted_or_removed(orig_comment.comment.id, context.pool()).await?;

        if user.id != orig_comment.comment.creator_id {
            return Err(TinyBoardsError::from_message(
                403,
                "comment edit not allowed",
            ));
        }

        let body = Some(data.body.clone());
        // we re-parse the markdown right here
        let body_html = parse_markdown(&body.clone().unwrap().as_str());
        let comment_id = path.comment_id;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let form = CommentForm {
            creator_id: orig_comment.comment.creator_id,
            post_id: orig_comment.comment.post_id,
            body,
            body_html,
            updated,
            ..CommentForm::default()
        };

        blocking(context.pool(), move |conn| {
            Comment::update(conn, comment_id, &form)
        })
        .await??;

        // do mentions parsing here
        // do mentions notifications sending here

        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(orig_comment.comment.creator_id))
        })
        .await??;

        Ok(CommentResponse { comment_view })
    }
}

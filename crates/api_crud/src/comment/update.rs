use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    build_response::build_comment_response,
    comment::{CommentIdPath, CommentResponse, EditComment},
    data::TinyBoardsContext,
    utils::{
        check_board_deleted_or_removed, check_comment_deleted_or_removed,
        check_post_deleted_or_removed, require_user,
    },
    websocket::send::send_notifications,
};
use tinyboards_db::{
    models::comment::comments::{Comment, CommentForm},
    traits::Crud,
    utils::naive_now,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::{
    error::TinyBoardsError,
    parser::parse_markdown_opt,
    utils::{custom_body_parsing, scrape_text_for_mentions},
};

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
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        let comment_id = path.comment_id;
        let orig_comment = CommentView::read(context.pool(), comment_id, None).await?;

        check_board_deleted_or_removed(orig_comment.board.id, context.pool()).await?;

        check_post_deleted_or_removed(orig_comment.post.id, context.pool()).await?;

        check_comment_deleted_or_removed(orig_comment.comment.id, context.pool()).await?;

        if view.person.id != orig_comment.comment.creator_id {
            return Err(TinyBoardsError::from_message(
                403,
                "comment edit not allowed",
            ));
        }

        let body = Some(data.body.clone());
        // we re-parse the markdown right here
        let mut body_html = parse_markdown_opt(&body.clone().unwrap().as_str());
        body_html = Some(custom_body_parsing(
            &body_html.unwrap_or_default(),
            context.settings(),
        ));

        let comment_id = path.comment_id;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let form = CommentForm {
            creator_id: Some(orig_comment.comment.creator_id),
            post_id: Some(orig_comment.comment.post_id),
            body,
            body_html,
            updated,
            ..CommentForm::default()
        };

        let updated_comment = Comment::update(context.pool(), comment_id, &form).await?;

        let comment_view = CommentView::read(
            context.pool(),
            comment_id,
            Some(orig_comment.comment.creator_id),
        )
        .await?;

        let mentions = scrape_text_for_mentions(&comment_view.comment.body_html);
        let recipient_ids = send_notifications(
            mentions,
            &comment_view.comment,
            &view.person,
            &orig_comment.post,
            context,
        )
        .await?;

        Ok(
            build_comment_response(context, updated_comment.id, Some(view), recipient_ids)
                .await?,
        )
    }
}

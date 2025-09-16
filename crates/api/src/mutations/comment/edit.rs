use crate::structs::comment::Comment;
use crate::DbPool;
use crate::LoggedInUser;
use crate::Settings;
use async_graphql::*;
use tinyboards_db::utils::naive_now;

use tinyboards_db::models::{
    board::boards::Board as DbBoard,
    comment::comments::{Comment as DbComment, CommentForm},
};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{parser::parse_markdown_opt, utils::custom_body_parsing, TinyBoardsError};

#[derive(Default)]
pub struct EditComment;

#[Object]
impl EditComment {
    pub async fn edit_comment(&self, ctx: &Context<'_>, id: i32, body: String) -> Result<Comment> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let comment = DbComment::read(pool, id).await?;
        // you can only edit your own content.
        if v.id != comment.creator_id {
            return Err(TinyBoardsError::from_message(403, "bruh").into());
        }

        if comment.is_deleted || comment.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "Your comment has been deleted or removed.",
            )
            .into());
        }

        let board = DbBoard::read(pool, comment.board_id).await?;
        // board mustn't be banned
        if board.is_removed {
            return Err(TinyBoardsError::from_message(
                410,
                &format!(
                    "+{} is banned. If you wish, you can delete your comment.",
                    &board.name
                ),
            )
            .into());
        }

        if board.is_banned {
            let reason = board.public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        // we need to re-parse the markdown here
        let mut body_html = parse_markdown_opt(body.as_str());
        body_html = Some(custom_body_parsing(
            &body_html.unwrap_or_default(),
            settings,
        ));

        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let form = CommentForm {
            body: Some(body),
            body_html,
            updated,
            ..CommentForm::default()
        };

        let _ = DbComment::update(pool, id, &form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update comment"))?;

        let res = DbComment::get_with_counts(pool, id).await?;

        Ok(Comment::from(res))
    }
}

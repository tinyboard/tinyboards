use crate::structs::comment::Comment;
use crate::DbPool;
use crate::LoggedInUser;
use crate::Settings;
use crate::utils::emoji::process_content_with_emojis;
use async_graphql::*;
use tinyboards_db::utils::naive_now;

use tinyboards_db::models::{
    board::boards::Board as DbBoard,
    comment::comments::{Comment as DbComment, CommentForm},
    post::posts::Post as DbPost,
};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{parser::{parse_markdown_opt, sanitize_html}, utils::custom_body_parsing, TinyBoardsError};

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
                    "/b/{} is banned. If you wish, you can delete your comment.",
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

        // Load site configuration and parent post to determine comment type
        let site_config = tinyboards_db::models::site::site::Site::read(pool).await?;
        let parent_post = DbPost::read(pool, comment.post_id).await?;

        // Thread comments use rich text editor (HTML), feed comments use markdown
        let is_thread_comment = parent_post.post_type == "thread";

        // Parse body with proper handling for thread vs feed comments
        let body_html = if is_thread_comment {
            // Thread comments use rich text editor - sanitize HTML directly
            let sanitized = sanitize_html(&body);
            let processed = if site_config.emoji_enabled {
                let emoji_limit = site_config.max_emojis_per_comment.map(|limit| limit as usize);
                process_content_with_emojis(
                    &sanitized,
                    pool,
                    Some(board.id),
                    settings,
                    emoji_limit,
                )
                .await?
            } else {
                custom_body_parsing(&sanitized, settings)
            };
            Some(processed)
        } else {
            // Feed comments use markdown - convert then sanitize
            if site_config.emoji_enabled {
                let emoji_limit = site_config.max_emojis_per_comment.map(|limit| limit as usize);
                let processed = process_content_with_emojis(
                    &body,
                    pool,
                    Some(board.id),
                    settings,
                    emoji_limit,
                )
                .await?;
                Some(sanitize_html(&processed))
            } else {
                let mut body_html = parse_markdown_opt(&body);
                body_html = Some(custom_body_parsing(
                    &body_html.unwrap_or_default(),
                    settings,
                ));
                body_html.map(|h| sanitize_html(&h))
            }
        };

        // Clean up orphaned uploads - images removed from the new HTML
        let storage = ctx.data::<crate::storage::StorageBackend>()?;
        if let Some(ref html) = body_html {
            if let Err(e) = crate::helpers::files::cleanup::cleanup_orphaned_uploads(
                pool,
                id,
                false, // is_comment
                html,
                storage,
            ).await {
                tracing::error!("Failed to cleanup orphaned uploads: {:?}", e);
                // Don't fail the edit if cleanup fails
            }
        }

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

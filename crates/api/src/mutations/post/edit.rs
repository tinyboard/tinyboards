use crate::structs::post::Post;
use crate::DbPool;
use crate::LoggedInUser;
use crate::Settings;
use crate::utils::emoji::process_content_with_emojis;
use async_graphql::*;
use tinyboards_db::utils::naive_now;

use tinyboards_db::models::{
    board::boards::Board as DbBoard,
    post::posts::{Post as DbPost, PostForm},
};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{parser::{parse_markdown_opt, sanitize_html}, utils::custom_body_parsing, TinyBoardsError};

#[derive(Default)]
pub struct EditPost;

#[Object]
impl EditPost {
    pub async fn edit_post(&self, ctx: &Context<'_>, id: i32, body: String, alt_text: Option<String>) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let post = DbPost::read(pool, id).await?;
        // you can only edit your own content.
        if v.id != post.creator_id {
            return Err(TinyBoardsError::from_message(403, "bruh").into());
        }

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "Your post has been deleted or removed.",
            )
            .into());
        }

        let board = DbBoard::read(pool, post.board_id).await?;
        // board mustn't be banned
        if board.is_removed {
            return Err(TinyBoardsError::from_message(
                410,
                &format!(
                    "/b/{} is banned. If you wish, you can delete your post.",
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

        // Load site configuration for emoji settings
        let site_config = tinyboards_db::models::site::site::Site::read(pool).await?;

        // Determine if this is a thread post (HTML from rich editor) or feed post (markdown)
        let is_thread_post = post.post_type == "thread";

        // Parse body with proper handling for thread vs feed posts
        let body_html = if is_thread_post {
            // Thread posts use rich text editor - sanitize HTML directly
            let sanitized = sanitize_html(&body);
            let processed = if site_config.emoji_enabled {
                let emoji_limit = site_config.max_emojis_per_post.map(|limit| limit as usize);
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
            // Feed posts use markdown - convert then sanitize
            if site_config.emoji_enabled {
                let emoji_limit = site_config.max_emojis_per_post.map(|limit| limit as usize);
                let processed_html = process_content_with_emojis(
                    &body,
                    pool,
                    Some(board.id),
                    settings,
                    emoji_limit,
                )
                .await?;
                Some(sanitize_html(&processed_html))
            } else {
                let body_html = parse_markdown_opt(&body);
                let processed = custom_body_parsing(
                    &body_html.unwrap_or_default(),
                    settings,
                );
                Some(sanitize_html(&processed))
            }
        };

        // Clean up orphaned uploads - images removed from the new HTML
        let storage = ctx.data::<crate::storage::StorageBackend>()?;
        if let Some(ref html) = body_html {
            if let Err(e) = crate::helpers::files::cleanup::cleanup_orphaned_uploads(
                pool,
                id,
                true, // is_post
                html,
                storage,
            ).await {
                tracing::error!("Failed to cleanup orphaned uploads: {:?}", e);
                // Don't fail the edit if cleanup fails
            }
        }

        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let form = PostForm {
            body: Some(body),
            body_html,
            updated,
            alt_text: alt_text,
            ..PostForm::default()
        };

        let _ = DbPost::update(pool, id, &form)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not update post"))?;

        let res = DbPost::get_with_counts(pool, id, false).await?;

        Ok(Post::from(res))
    }
}

use crate::helpers::permissions;
use crate::structs::comment::Comment;
use crate::utils::emoji::process_content_with_emojis;
use crate::{DbPool, Settings};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::CommentAggregates,
        board::boards::Board as DbBoard,
        comment::comments::{Comment as DbComment, CommentUpdateForm},
        site::site::Site,
    },
    schema::{boards, comment_aggregates, comments, site},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct EditComment;

#[Object]
impl EditComment {
    pub async fn edit_comment(&self, ctx: &Context<'_>, id: ID, body: String) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        let comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        if user.id != comment.creator_id {
            return Err(
                TinyBoardsError::from_message(403, "You can only edit your own comments").into(),
            );
        }

        if comment.deleted_at.is_some() || comment.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "Your comment has been deleted or removed.",
            )
            .into());
        }

        let board: DbBoard = boards::table
            .find(comment.board_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".into()))?;

        if board.is_banned {
            let reason = board
                .public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        // Load site config for emoji settings
        let site_config: Site = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let emoji_limit = if site_config.emoji_enabled {
            site_config
                .max_emojis_per_post
                .map(|limit| limit as usize)
        } else {
            Some(0)
        };

        let body_html = Some(
            process_content_with_emojis(&body, pool, Some(board.id), settings, emoji_limit)
                .await?,
        );

        // Clean up orphaned uploads
        let storage = ctx.data::<crate::storage::StorageBackend>()?;
        if let Some(ref html) = body_html {
            if let Err(e) = crate::helpers::files::cleanup::cleanup_orphaned_uploads(
                pool,
                comment_uuid,
                false,
                html,
                storage,
            )
            .await
            {
                tracing::error!("Failed to cleanup orphaned uploads: {:?}", e);
            }
        }

        let form = CommentUpdateForm {
            body: Some(body),
            body_html,
            updated_at: Some(chrono::Utc::now()),
            ..CommentUpdateForm::default()
        };

        diesel::update(comments::table.find(comment_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let db_comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;
        let agg: CommentAggregates = comment_aggregates::table
            .filter(comment_aggregates::comment_id.eq(comment_uuid))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment aggregates not found".into()))?;

        Ok(Comment::from((db_comment, agg)))
    }
}

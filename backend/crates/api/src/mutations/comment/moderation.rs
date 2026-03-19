use crate::helpers::{permissions, validation::require_mod_or_admin};
use crate::structs::comment::Comment;
use crate::DbPool;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbApprovalStatus, DbModerationAction},
    models::{
        aggregates::CommentAggregates,
        board::board_mods::ModPerms,
        comment::comments::{Comment as DbComment, CommentUpdateForm},
        moderator::moderation_log::ModerationLogInsertForm,
        user::user::AdminPerms,
    },
    schema::{comment_aggregates, comments, moderation_log},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct CommentModeration;

/// Helper to load a comment with its aggregates
async fn load_comment_with_counts(
    conn: &mut diesel_async::AsyncPgConnection,
    comment_id: Uuid,
) -> Result<Comment, TinyBoardsError> {
    let db_comment: DbComment = comments::table
        .find(comment_id)
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;
    let agg: CommentAggregates = comment_aggregates::table
        .filter(comment_aggregates::comment_id.eq(comment_id))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::NotFound("Comment aggregates not found".into()))?;
    Ok(Comment::from((db_comment, agg)))
}

#[Object]
impl CommentModeration {
    /// Remove a comment (mod/admin action)
    pub async fn remove_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        reason: Option<String>,
    ) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        let comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        require_mod_or_admin(
            user,
            pool,
            comment.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        diesel::update(comments::table.find(comment_uuid))
            .set(&CommentUpdateForm {
                is_removed: Some(true),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log to moderation_log
        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RemoveComment,
                target_type: "comment".to_string(),
                target_id: comment_uuid,
                board_id: Some(comment.board_id),
                reason,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Approve a comment (mod/admin action)
    pub async fn approve_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        let comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        require_mod_or_admin(
            user,
            pool,
            comment.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        diesel::update(comments::table.find(comment_uuid))
            .set(&CommentUpdateForm {
                is_removed: Some(false),
                approval_status: Some(DbApprovalStatus::Approved),
                approved_by: Some(Some(user.id)),
                approved_at: Some(Some(chrono::Utc::now())),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RestoreComment,
                target_type: "comment".to_string(),
                target_id: comment_uuid,
                board_id: Some(comment.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Restore a removed comment (alias for approve_comment, used by frontend)
    pub async fn restore_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
        self.approve_comment(ctx, comment_id).await
    }

    /// Pin a comment (mod/admin action)
    pub async fn pin_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        let comment: DbComment = comments::table
            .find(comment_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Comment not found".into()))?;

        require_mod_or_admin(
            user,
            pool,
            comment.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        let new_pinned = !comment.is_pinned;

        diesel::update(comments::table.find(comment_uuid))
            .set(&CommentUpdateForm {
                is_pinned: Some(new_pinned),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }
}

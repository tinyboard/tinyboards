use crate::helpers::permissions;
use crate::structs::comment::Comment;
use crate::DbPool;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::CommentAggregates,
        board::board_mods::{BoardModerator, ModPerms},
        comment::comment_votes::CommentVoteInsertForm,
        comment::comments::{Comment as DbComment, CommentUpdateForm},
        post::posts::Post as DbPost,
        social::CommentSavedInsertForm,
        user::user::AdminPerms,
    },
    schema::{
        board_moderators, board_user_bans, comment_aggregates, comment_saved, comment_votes,
        comments, posts,
    },
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct CommentActions;

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
impl CommentActions {
    pub async fn vote_on_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        #[graphql(desc = "1 for upvote, -1 for downvote, 0 to remove vote")] direction: i32,
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

        if comment.deleted_at.is_some() || comment.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "Comment has been deleted or removed.",
            )
            .into());
        }

        // Check if user is banned from board
        let banned: bool = board_user_bans::table
            .filter(board_user_bans::board_id.eq(comment.board_id))
            .filter(board_user_bans::user_id.eq(user.id))
            .first::<tinyboards_db::models::social::BoardUserBan>(conn)
            .await
            .is_ok();

        if banned {
            return load_comment_with_counts(conn, comment_uuid)
                .await
                .map_err(|e| e.into());
        }

        // Remove any existing vote
        diesel::delete(
            comment_votes::table
                .filter(comment_votes::user_id.eq(user.id))
                .filter(comment_votes::comment_id.eq(comment_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Insert new vote if direction is not 0
        if direction == 1 || direction == -1 {
            let vote_form = CommentVoteInsertForm {
                id: Uuid::new_v4(),
                user_id: user.id,
                comment_id: comment_uuid,
                post_id: comment.post_id,
                score: direction as i16,
            };
            diesel::insert_into(comment_votes::table)
                .values(&vote_form)
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Save a comment
    pub async fn save_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        let form = CommentSavedInsertForm {
            comment_id: comment_uuid,
            user_id: user.id,
        };

        diesel::insert_into(comment_saved::table)
            .values(&form)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Unsave a comment
    pub async fn unsave_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let comment_uuid: Uuid = comment_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid comment ID"))?;

        diesel::delete(
            comment_saved::table
                .filter(comment_saved::comment_id.eq(comment_uuid))
                .filter(comment_saved::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Delete a comment (soft delete, creator or admin)
    pub async fn delete_comment(&self, ctx: &Context<'_>, comment_id: ID) -> Result<Comment> {
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

        let can_delete = comment.creator_id == user.id
            || user.has_permission(AdminPerms::Content)
            || board_moderators::table
                .filter(board_moderators::board_id.eq(comment.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .first::<BoardModerator>(conn)
                .await
                .ok()
                .map(|m| m.has_permission(ModPerms::Content))
                .unwrap_or(false);

        if !can_delete {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to delete this comment",
            )
            .into());
        }

        let form = CommentUpdateForm {
            deleted_at: Some(Some(chrono::Utc::now())),
            ..Default::default()
        };

        diesel::update(comments::table.find(comment_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_comment_with_counts(conn, comment_uuid)
            .await
            .map_err(|e| e.into())
    }
}

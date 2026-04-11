use crate::helpers::{permissions, validation::require_mod_or_admin};
use crate::structs::post::Post;
use crate::DbPool;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbApprovalStatus, DbModerationAction},
    models::{
        aggregates::PostAggregates,
        board::board_mods::ModPerms,
        moderator::moderation_log::ModerationLogInsertForm,
        post::posts::{Post as DbPost, PostUpdateForm},
        user::user::AdminPerms,
    },
    schema::{moderation_log, post_aggregates, posts},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct PostModeration;

/// Helper to load a post with its aggregates
async fn load_post_with_counts(
    conn: &mut diesel_async::AsyncPgConnection,
    post_id: Uuid,
) -> Result<Post, TinyBoardsError> {
    let db_post: DbPost = posts::table
        .find(post_id)
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;
    let agg: PostAggregates = post_aggregates::table
        .filter(post_aggregates::post_id.eq(post_id))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::NotFound("Post aggregates not found".into()))?;
    Ok(Post::from((db_post, agg)))
}

#[Object]
impl PostModeration {
    /// Remove a post (mod/admin action)
    pub async fn remove_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        reason: Option<String>,
    ) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
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
                action_type: DbModerationAction::RemovePost,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Approve a post (mod/admin action)
    pub async fn approve_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                is_removed: Some(false),
                approval_status: Some(DbApprovalStatus::Approved),
                approved_by: Some(Some(user.id)),
                approved_at: Some(Some(chrono::Utc::now())),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Log to moderation_log
        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::RestorePost,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Restore a removed post (alias for approve_post, used by frontend)
    pub async fn restore_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        self.approve_post(ctx, post_id).await
    }

    /// Lock a post (mod/admin action)
    pub async fn lock_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                is_locked: Some(true),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::LockPost,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Unlock a post (mod/admin action)
    pub async fn unlock_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                is_locked: Some(false),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::UnlockPost,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Toggle distinguish on a post (mark as speaking officially as admin or mod).
    /// Only the post creator can distinguish, and only if they are an admin or mod of the board.
    pub async fn distinguish_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
    ) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        // Only the post creator can distinguish their own posts
        if post.creator_id != user.id {
            return Err(
                TinyBoardsError::from_message(403, "Only the post creator can distinguish a post")
                    .into(),
            );
        }

        // Determine the distinguish type based on the user's role
        let new_distinguished = if post.distinguished_as.is_some() {
            // Toggle off
            None
        } else if user.is_admin && user.admin_level > 0 {
            Some("admin".to_string())
        } else {
            // Check if user is a mod of this board
            use tinyboards_db::schema::board_moderators;
            let is_mod: bool = board_moderators::table
                .filter(board_moderators::board_id.eq(post.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .count()
                .get_result::<i64>(conn)
                .await
                .map(|c| c > 0)
                .unwrap_or(false);

            if is_mod {
                Some("mod".to_string())
            } else {
                return Err(
                    TinyBoardsError::from_message(403, "Must be an admin or board moderator to distinguish")
                        .into(),
                );
            }
        };

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                distinguished_as: Some(new_distinguished),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Mark a post as NSFW (mod/admin action)
    pub async fn mark_nsfw_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                is_nsfw: Some(true),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::MarkNsfw,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Unmark a post as NSFW (mod/admin action)
    pub async fn unmark_nsfw_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        require_mod_or_admin(user, pool, post.board_id, ModPerms::Content, Some(AdminPerms::Content))
            .await?;

        diesel::update(posts::table.find(post_uuid))
            .set(&PostUpdateForm {
                is_nsfw: Some(false),
                ..Default::default()
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::insert_into(moderation_log::table)
            .values(&ModerationLogInsertForm {
                moderator_id: user.id,
                action_type: DbModerationAction::UnmarkNsfw,
                target_type: "post".to_string(),
                target_id: post_uuid,
                board_id: Some(post.board_id),
                reason: None,
                metadata: None,
                expires_at: None,
            })
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }
}

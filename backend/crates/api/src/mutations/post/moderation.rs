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
}

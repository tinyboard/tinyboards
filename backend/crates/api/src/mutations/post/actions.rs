use crate::helpers::permissions;
use crate::structs::post::Post;
use crate::DbPool;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::PostAggregates,
        board::board_mods::{BoardModerator, ModPerms},
        post::post_votes::PostVoteInsertForm,
        post::posts::{Post as DbPost, PostUpdateForm},
        social::PostHiddenInsertForm,
        social::PostSavedInsertForm,
        user::user::AdminPerms,
    },
    schema::{
        board_moderators, board_user_bans, boards, post_aggregates, post_hidden, post_saved,
        post_votes, posts,
    },
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct PostActions;

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
impl PostActions {
    pub async fn vote_on_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        #[graphql(desc = "1 for upvote, -1 for downvote, 0 to remove vote")] direction: i32,
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

        if post.deleted_at.is_some() || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        // Check board is not banned
        let board: tinyboards_db::models::board::boards::Board = boards::table
            .find(post.board_id)
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

        // Check if user is banned from board
        let is_banned_from_board: bool = board_user_bans::table
            .filter(board_user_bans::board_id.eq(board.id))
            .filter(board_user_bans::user_id.eq(user.id))
            .first::<tinyboards_db::models::social::BoardUserBan>(conn)
            .await
            .is_ok();

        if is_banned_from_board {
            return load_post_with_counts(conn, post_uuid)
                .await
                .map_err(|e| e.into());
        }

        // Remove any existing vote
        diesel::delete(
            post_votes::table
                .filter(post_votes::user_id.eq(user.id))
                .filter(post_votes::post_id.eq(post_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Insert new vote if direction is not 0
        if direction == 1 || direction == -1 {
            let vote_form = PostVoteInsertForm {
                id: Uuid::new_v4(),
                user_id: user.id,
                post_id: post_uuid,
                score: direction as i16,
            };
            diesel::insert_into(post_votes::table)
                .values(&vote_form)
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Save a post
    pub async fn save_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let form = PostSavedInsertForm {
            post_id: post_uuid,
            user_id: user.id,
        };

        diesel::insert_into(post_saved::table)
            .values(&form)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Unsave a post
    pub async fn unsave_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        diesel::delete(
            post_saved::table
                .filter(post_saved::post_id.eq(post_uuid))
                .filter(post_saved::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Hide a post
    pub async fn hide_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<bool> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let form = PostHiddenInsertForm {
            post_id: post_uuid,
            user_id: user.id,
        };

        diesel::insert_into(post_hidden::table)
            .values(&form)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Unhide a post
    pub async fn unhide_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<bool> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = post_id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        diesel::delete(
            post_hidden::table
                .filter(post_hidden::post_id.eq(post_uuid))
                .filter(post_hidden::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Delete a post (soft delete, creator or admin)
    pub async fn delete_post(&self, ctx: &Context<'_>, post_id: ID) -> Result<Post> {
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

        let can_delete = post.creator_id == user.id
            || user.has_permission(AdminPerms::Content)
            || board_moderators::table
                .filter(board_moderators::board_id.eq(post.board_id))
                .filter(board_moderators::user_id.eq(user.id))
                .first::<BoardModerator>(conn)
                .await
                .ok()
                .map(|m| m.has_permission(ModPerms::Content))
                .unwrap_or(false);

        if !can_delete {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to delete this post",
            )
            .into());
        }

        // Clean up associated files
        let storage = ctx.data::<crate::storage::StorageBackend>()?;
        if let Err(e) =
            crate::helpers::files::cleanup::delete_post_files(pool, post_uuid, storage).await
        {
            tracing::error!("Failed to cleanup post files: {:?}", e);
        }

        let form = PostUpdateForm {
            deleted_at: Some(Some(chrono::Utc::now())),
            ..Default::default()
        };

        diesel::update(posts::table.find(post_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }

    /// Feature or unfeature a post (moderator/admin action)
    pub async fn feature_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        featured: bool,
        #[graphql(desc = "\"local\" or \"board\"")] feature_type: Option<String>,
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

        let feature_type = feature_type.unwrap_or_else(|| "board".to_string());

        let can_feature = match feature_type.as_str() {
            "local" => user.has_permission(AdminPerms::Content),
            "board" => {
                if user.has_permission(AdminPerms::Content) {
                    true
                } else {
                    board_moderators::table
                        .filter(board_moderators::board_id.eq(post.board_id))
                        .filter(board_moderators::user_id.eq(user.id))
                        .first::<BoardModerator>(conn)
                        .await
                        .ok()
                        .map(|m| m.has_permission(ModPerms::Content))
                        .unwrap_or(false)
                }
            }
            _ => false,
        };

        if !can_feature {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to feature posts",
            )
            .into());
        }

        let form = PostUpdateForm {
            is_featured_local: if feature_type == "local" {
                Some(featured)
            } else {
                None
            },
            is_featured_board: if feature_type == "board" {
                Some(featured)
            } else {
                None
            },
            ..Default::default()
        };

        diesel::update(posts::table.find(post_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        load_post_with_counts(conn, post_uuid)
            .await
            .map_err(|e| e.into())
    }
}

/**
 * Post actions that do not require too much code, such as:
 *  - Mod actions: remove, approve, pin
 *  - Delete
 *  - Vote
 **/
use crate::structs::post::Post;
use crate::DbPool;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::models::board::boards::Board as DbBoard;
use tinyboards_db::models::board::board_mods::{BoardModerator, ModPerms};
use tinyboards_db::models::post::post_saved::{PostSaved, PostSavedForm};
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_db::models::post::post_votes::PostVoteForm;
use tinyboards_db::models::post::posts::{Post as DbPost, PostForm};
use tinyboards_db::models::post::post_hidden::{PostHidden, PostHiddenForm};
use tinyboards_db::traits::{Crud, Saveable, Voteable};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct PostActions;

#[Object]
impl PostActions {
    pub async fn vote_on_post(&self, ctx: &Context<'_>, id: i32, vote_type: i32) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let post = DbPost::read(pool, id).await?;

        // Prevent voting on thread posts
        if post.post_type == "thread" {
            return Err(TinyBoardsError::from_message(
                400,
                "Voting is not enabled for thread posts. Thread posts are sorted by activity instead.",
            )
            .into());
        }

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        let board = DbBoard::read(pool, post.board_id).await?;
        // board mustn't be banned
        if board.is_removed {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("/b/{} is banned.", &board.name),
            )
            .into());
        }

        if board.is_banned {
            let reason = board.public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        let is_banned_from_board = DbBoard::board_has_ban(pool, board.id, v.id)
            .await
            .unwrap_or(true);

        // vote is not registered if the user is banned from the board
        if !is_banned_from_board {
            // remove any existing votes first
            DbPostVote::remove(pool, v.id, post.id).await?;

            // if vote type is 0, only remove the user's existing vote
            // otherwise register the new vote
            let do_add = vote_type != 0 && (vote_type == 1 || vote_type == -1);

            if do_add {
                let vote_form = PostVoteForm {
                    post_id: post.id,
                    user_id: v.id,
                    score: vote_type,
                };

                DbPostVote::vote(pool, &vote_form).await?;
            }
        }

        let res = DbPost::get_with_counts(pool, post.id, false).await?;

        Ok(Post::from(res))
    }

    /// Save or unsave a post
    pub async fn save_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        save: bool,
    ) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let post = DbPost::read(pool, post_id).await?;

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        let form = PostSavedForm {
            post_id,
            user_id: user.id,
        };

        if save {
            PostSaved::save(pool, &form).await?;
        } else {
            PostSaved::unsave(pool, &form).await?;
        }

        let res = DbPost::get_with_counts(pool, post_id, false).await?;
        Ok(Post::from(res))
    }

    /// Feature or unfeature a post (moderator/admin action)
    pub async fn feature_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        featured: bool,
        feature_type: Option<String>, // "local" or "board" 
    ) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let post = DbPost::read(pool, post_id).await?;
        let _board = DbBoard::read(pool, post.board_id).await?;

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        let feature_type = feature_type.unwrap_or_else(|| "board".to_string());

        // For thread posts, only allow board-level featuring (pinning)
        if post.post_type == "thread" && feature_type == "local" {
            return Err(TinyBoardsError::from_message(
                400,
                "Thread posts can only be pinned to their board, not featured site-wide.",
            )
            .into());
        }

        // Check permissions
        let can_feature = match feature_type.as_str() {
            "local" => {
                // Only admins can feature locally (site-wide)
                // System, Owner, Full, or Content level admins can feature locally
                user.has_permission(tinyboards_db::models::user::user::AdminPerms::Content) ||
                user.has_permission(tinyboards_db::models::user::user::AdminPerms::Full) ||
                user.has_permission(tinyboards_db::models::user::user::AdminPerms::Owner) ||
                user.has_permission(tinyboards_db::models::user::user::AdminPerms::System)
            }
            "board" => {
                // Moderators with content permissions can feature in board
                // Higher-level admins (Content, Full, Owner, System) can always feature
                if user.has_permission(tinyboards_db::models::user::user::AdminPerms::Content) ||
                   user.has_permission(tinyboards_db::models::user::user::AdminPerms::Full) ||
                   user.has_permission(tinyboards_db::models::user::user::AdminPerms::Owner) ||
                   user.has_permission(tinyboards_db::models::user::user::AdminPerms::System) {
                    true // Admins can always feature
                } else {
                    // Check if user is a moderator of this board with content permissions
                    match BoardModerator::get_by_user_id_for_board(pool, user.id, post.board_id, true).await {
                        Ok(moderator) => moderator.has_permission(ModPerms::Content),
                        Err(_) => false, // User is not a moderator
                    }
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

        // Update the post featuring
        let form = PostForm {
            featured_local: if feature_type == "local" { Some(featured) } else { None },
            featured_board: if feature_type == "board" { Some(featured) } else { None },
            ..Default::default()
        };

        DbPost::update(pool, post_id, &form).await?;
        
        let res = DbPost::get_with_counts(pool, post_id, false).await?;
        Ok(Post::from(res))
    }

    /// Delete a post (creator/moderator/admin only)
    pub async fn delete_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        deleted: bool,
    ) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let post = DbPost::read(pool, post_id).await?;

        // Check permissions: creator can delete their own post, or moderator/admin
        let can_delete = post.creator_id == user.id ||
            user.has_permission(tinyboards_db::models::user::user::AdminPerms::Content) ||
            match BoardModerator::get_by_user_id_for_board(pool, user.id, post.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            };

        if !can_delete {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to delete this post",
            )
            .into());
        }

        // If actually deleting (not undeleting), clean up associated files
        if deleted {
            let storage = ctx.data::<crate::storage::StorageBackend>()?;
            if let Err(e) = crate::helpers::files::cleanup::delete_post_files(pool, post_id, storage).await {
                tracing::error!("Failed to cleanup post files: {:?}", e);
                // Don't fail the deletion if file cleanup fails
            }
        }

        // Update the post's deleted status
        DbPost::update_deleted(pool, post_id, deleted).await?;

        let res = DbPost::get_with_counts(pool, post_id, false).await?;
        Ok(Post::from(res))
    }

    /// Hide or unhide a post (user action)
    pub async fn hide_post(
        &self,
        ctx: &Context<'_>,
        post_id: i32,
        hidden: bool,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let post = DbPost::read(pool, post_id).await?;

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        // Hide/unhide post for this user
        let form = PostHiddenForm {
            post_id,
            user_id: user.id,
        };

        if hidden {
            PostHidden::hide(pool, &form).await?;
        } else {
            PostHidden::unhide(pool, &form).await?;
        }

        Ok(hidden)
    }

    // Note: Post reporting is implemented in mutations/reports.rs
}

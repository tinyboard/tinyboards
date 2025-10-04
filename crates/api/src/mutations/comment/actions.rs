/**
 * Comment actions that do not require too much code, such as:
 *  - Mod actions: remove, approve, pin
 *  - Delete
 *  - Vote
 **/
use crate::structs::comment::Comment;
use crate::DbPool;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::models::board::boards::Board as DbBoard;
use tinyboards_db::models::board::board_mods::{BoardModerator, ModPerms};
use tinyboards_db::models::comment::comment_saved::{CommentSaved, CommentSavedForm};
use tinyboards_db::models::comment::comment_votes::CommentVote as DbCommentVote;
use tinyboards_db::models::comment::comment_votes::CommentVoteForm;
use tinyboards_db::models::comment::comments::Comment as DbComment;
use tinyboards_db::traits::{Crud, Saveable, Voteable};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct CommentActions;

#[Object]
impl CommentActions {
    pub async fn vote_on_comment(
        &self,
        ctx: &Context<'_>,
        id: i32,
        vote_type: i32,
    ) -> Result<Comment> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let comment = DbComment::read(pool, id).await?;

        if comment.is_deleted || comment.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That comment has been deleted or removed.",
            )
            .into());
        }

        let board = DbBoard::read(pool, comment.board_id).await?;
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
            DbCommentVote::remove(pool, v.id, comment.id).await?;

            // if vote type is 0, only remove the user's existing vote
            // otherwise register the new vote
            let do_add = vote_type != 0 && (vote_type == 1 || vote_type == -1);

            if do_add {
                let vote_form = CommentVoteForm {
                    comment_id: comment.id,
                    user_id: v.id,
                    post_id: comment.post_id,
                    score: vote_type,
                };

                DbCommentVote::vote(pool, &vote_form).await?;
            }
        }

        let res = DbComment::get_with_counts(pool, comment.id).await?;

        Ok(Comment::from(res))
    }

    /// Save or unsave a comment
    pub async fn save_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: i32,
        save: bool,
    ) -> Result<Comment> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let comment = DbComment::read(pool, comment_id).await?;

        if comment.is_deleted || comment.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That comment has been deleted or removed.",
            )
            .into());
        }

        let form = CommentSavedForm {
            comment_id,
            user_id: user.id,
        };

        if save {
            CommentSaved::save(pool, &form).await?;
        } else {
            CommentSaved::unsave(pool, &form).await?;
        }

        let res = DbComment::get_with_counts(pool, comment_id).await?;
        Ok(Comment::from(res))
    }

    /// Delete a comment (creator/moderator/admin only)
    pub async fn delete_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: i32,
        deleted: bool,
    ) -> Result<Comment> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_not_banned()?;

        let comment = DbComment::read(pool, comment_id).await?;

        // Check permissions: creator can delete their own comment, or moderator/admin
        let can_delete = comment.creator_id == user.id ||
            user.has_permission(tinyboards_db::models::user::user::AdminPerms::Content) ||
            match BoardModerator::get_by_user_id_for_board(pool, user.id, comment.board_id, true).await {
                Ok(moderator) => moderator.has_permission(ModPerms::Content),
                Err(_) => false,
            };

        if !can_delete {
            return Err(TinyBoardsError::from_message(
                403,
                "You don't have permission to delete this comment",
            )
            .into());
        }

        // If actually deleting (not undeleting), clean up associated files
        if deleted {
            let storage = ctx.data::<crate::storage::StorageBackend>()?;
            if let Err(e) = crate::helpers::files::cleanup::delete_comment_files(pool, comment_id, storage).await {
                tracing::error!("Failed to cleanup comment files: {:?}", e);
                // Don't fail the deletion if file cleanup fails
            }
        }

        // Update the comment's deleted status
        DbComment::update_deleted(pool, comment_id, deleted).await?;

        let res = DbComment::get_with_counts(pool, comment_id).await?;
        Ok(Comment::from(res))
    }

    // Note: Comment reporting is implemented in mutations/reports.rs
}

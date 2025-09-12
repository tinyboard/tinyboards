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
use tinyboards_db::models::post::post_saved::{PostSaved, PostSavedForm};
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_db::models::post::post_votes::PostVoteForm;
use tinyboards_db::models::post::posts::{Post as DbPost, PostForm};
use tinyboards_db::traits::{Crud, Saveable, Voteable};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct PostActions;

#[Object]
impl PostActions {
    pub async fn vote_on_post(&self, ctx: &Context<'_>, id: i32, vote_type: i16) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let post = DbPost::read(pool, id).await?;

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
                &format!("+{} is banned.", &board.name),
            )
            .into());
        }

        if board.is_banned {
            let reason = board.public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        let is_banned_from_board = DbBoard::board_has_ban(pool, board.id, v.person.id)
            .await
            .unwrap_or(true);

        // vote is not registered if the user is banned from the board
        if !is_banned_from_board {
            // remove any existing votes first
            DbPostVote::remove(pool, v.person.id, post.id).await?;

            // if vote type is 0, only remove the user's existing vote
            // otherwise register the new vote
            let do_add = vote_type != 0 && (vote_type == 1 || vote_type == -1);

            if do_add {
                let vote_form = PostVoteForm {
                    post_id: post.id,
                    person_id: v.person.id,
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
            person_id: user.person.id,
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
        let board = DbBoard::read(pool, post.board_id).await?;

        if post.is_deleted || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "That post has been deleted or removed.",
            )
            .into());
        }

        let feature_type = feature_type.unwrap_or_else(|| "board".to_string());
        
        // Check permissions
        let can_feature = match feature_type.as_str() {
            "local" => {
                // Only admins can feature locally (site-wide)
                user.has_permission(tinyboards_db::models::person::local_user::AdminPerms::Content)
            }
            "board" => {
                // Moderators with content permissions can feature in board
                if user.has_permission(tinyboards_db::models::person::local_user::AdminPerms::Content) {
                    true // Admins can always feature
                } else {
                    // Check if user is a moderator of this board with content permissions
                    // For now, we'll implement a basic check
                    false // TODO: Implement proper mod permission check
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

    // TODO: post reporting
}

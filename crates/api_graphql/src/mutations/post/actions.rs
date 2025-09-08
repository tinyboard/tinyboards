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
use tinyboards_db::models::post::post_votes::PostVote as DbPostVote;
use tinyboards_db::models::post::post_votes::PostVoteForm;
use tinyboards_db::models::post::posts::Post as DbPost;
use tinyboards_db::traits::Crud;
use tinyboards_db::traits::Voteable;
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

    // TODO: post reporting
}

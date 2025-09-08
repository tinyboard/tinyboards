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
use tinyboards_db::models::comment::comment_votes::CommentVote as DbCommentVote;
use tinyboards_db::models::comment::comment_votes::CommentVoteForm;
use tinyboards_db::models::comment::comments::Comment as DbComment;
use tinyboards_db::traits::Crud;
use tinyboards_db::traits::Voteable;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct CommentActions;

#[Object]
impl CommentActions {
    pub async fn vote_on_comment(
        &self,
        ctx: &Context<'_>,
        id: i32,
        vote_type: i16,
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
            DbCommentVote::remove(pool, v.person.id, comment.id).await?;

            // if vote type is 0, only remove the user's existing vote
            // otherwise register the new vote
            let do_add = vote_type != 0 && (vote_type == 1 || vote_type == -1);

            if do_add {
                let vote_form = CommentVoteForm {
                    comment_id: comment.id,
                    person_id: v.person.id,
                    post_id: comment.post_id,
                    score: vote_type,
                };

                DbCommentVote::vote(pool, &vote_form).await?;
            }
        }

        let res = DbComment::get_with_counts(pool, comment.id).await?;

        Ok(Comment::from(res))
    }

    // TODO: comment reporting
}

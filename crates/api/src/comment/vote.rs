use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CreateCommentVote},
    data::TinyBoardsContext,
    site::Message,
    utils::{
        blocking, check_board_deleted_or_removed, check_comment_deleted_or_removed,
        check_post_deleted_removed_or_locked, require_user,
    },
};
use tinyboards_db::{
    models::{
        board::board::Board,
        comment::comment_vote::{CommentVote, CommentVoteForm},
    },
    traits::Voteable,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreateCommentVote {
    type Response = Message;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &CreateCommentVote = &self;
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        let comment_id = path.comment_id;

        let orig_comment = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(user.id))
        })
        .await??;

        // check to see if board is even there or not
        check_board_deleted_or_removed(orig_comment.post.board_id, context.pool()).await?;

        // check if post even exists or not (or if it is locked)
        check_post_deleted_removed_or_locked(orig_comment.post.id, context.pool()).await?;

        // check if comment even exists or not
        check_comment_deleted_or_removed(comment_id, context.pool()).await?;

        let is_board_banned = blocking(context.pool(), move |conn| {
            Board::board_has_ban(conn, orig_comment.post.board_id, user.id)
        })
        .await??;

        if !is_board_banned {
            let vote_form = CommentVoteForm {
                comment_id: path.comment_id,
                user_id: user.id,
                score: data.score,
            };

            // remove any existing votes first
            blocking(context.pool(), move |conn| {
                CommentVote::remove(conn, user.id, comment_id)
            })
            .await??;

            let do_add = vote_form.score != 0 && (vote_form.score == 1 || vote_form.score == -1);

            if do_add {
                let cloned_form = vote_form.clone();
                let like = move |conn: &mut _| CommentVote::vote(conn, &cloned_form);
                blocking(context.pool(), like)
                    .await??;
            } else {
                let cloned_form = vote_form.clone();
                let like = move |conn: &mut _| {
                    CommentVote::remove(conn, cloned_form.user_id, cloned_form.comment_id)
                };
                blocking(context.pool(), like).await??;
            }
        }

        // mark comment as read here

        // grab updated comment view here
        /*let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, Some(user.id))
                .map_err(|_| TinyBoardsError::err_500())
        })
        .await??;*/

        Ok(Message::new("Vote saved!"))
    }
}

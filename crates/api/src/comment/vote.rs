use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{CommentIdPath, CommentResponse, CreateCommentVote},
    data::TinyBoardsContext,
    utils::{
        check_board_deleted_or_removed, check_comment_deleted_or_removed,
        check_post_deleted_removed_or_locked, require_user,
    },
};
use tinyboards_db::{
    models::{
        board::boards::Board,
        comment::comment_votes::{CommentVote, CommentVoteForm},
    },
    traits::Voteable,
};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreateCommentVote {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &CreateCommentVote = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        let comment_id = path.comment_id;

        let orig_comment = CommentView::read(context.pool(), comment_id, Some(view.person.id)).await?;

        // check to see if board is even there or not
        check_board_deleted_or_removed(orig_comment.post.board_id, context.pool()).await?;

        // check if post even exists or not (or if it is locked)
        check_post_deleted_removed_or_locked(orig_comment.post.id, context.pool()).await?;

        // check if comment even exists or not
        check_comment_deleted_or_removed(comment_id, context.pool()).await?;

        let is_board_banned = Board::board_has_ban(context.pool(), orig_comment.post.board_id, view.person.id).await?;

        if !is_board_banned {
            let vote_form = CommentVoteForm {
                comment_id: path.comment_id,
                person_id: view.person.id,
                score: data.score,
            };

            // remove any existing votes first
            CommentVote::remove(context.pool(), view.person.id, comment_id).await?;

            let do_add = vote_form.score != 0 && (vote_form.score == 1 || vote_form.score == -1);

            if do_add {
                let cloned_form = vote_form.clone();
                CommentVote::vote(context.pool(), &cloned_form).await?;
            } else {
                let cloned_form = vote_form.clone();
                CommentVote::remove(context.pool(), cloned_form.person_id, cloned_form.comment_id).await?;
            }
        }

        // mark comment as read here

        // grab updated comment view here
        let comment_view = CommentView::read(context.pool(), comment_id, Some(view.person.id)).await?;

        Ok(CommentResponse { comment_view })
    }
}

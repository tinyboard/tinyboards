use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{CreatePostVote, PostIdPath, PostResponse},
    utils::{
        check_board_deleted_or_removed, check_post_deleted_removed_or_locked,
        require_user,
    },
};
use tinyboards_db::{
    models::post::post_votes::{PostVote, PostVoteForm},
    models::{board::boards::Board, post::posts::Post},
    traits::{Crud, Voteable},
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreatePostVote {
    type Response = PostResponse;
    type Route = PostIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &CreatePostVote = &self;
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .unwrap()?;

        // check if downvotes are disabled (when/if we implement this feature)

        let post_id = path.post_id;

        let post = Post::read(context.pool(), post_id).await?;

        // check if post can be liked (not deleted, removed, or locked)
        check_post_deleted_removed_or_locked(post_id, context.pool()).await?;

        // check if the board is there or not
        check_board_deleted_or_removed(post.board_id, context.pool()).await?;

        // check if user is banned from board
        let is_board_banned = Board::board_has_ban(context.pool(), post.board_id, user.id).await?;

        if !is_board_banned {
            let vote_form = PostVoteForm {
                post_id: path.post_id,
                user_id: user.id,
                score: data.score,
            };

            // remove any existing votes first
            let user_id = user.id;
            PostVote::remove(context.pool(), user_id, post_id).await?;

            let do_add = vote_form.score != 0 && (vote_form.score == 1 || vote_form.score == -1);

            if do_add {
                let cloned_form = vote_form.clone();
                PostVote::vote(context.pool(), &cloned_form).await?;
            } else {
                let cloned_form = vote_form.clone();
                PostVote::remove(context.pool(), cloned_form.user_id, cloned_form.post_id).await?;
            }

            // mark the post as read here

            // grab the post view here for the response
            let post_view = PostView::read(context.pool(), vote_form.post_id, Some(vote_form.user_id)).await?;

            Ok(PostResponse { post_view })
        } else {
            Err(TinyBoardsError::from_message(
                403,
                "you are banned from the board",
            ))
        }
    }
}

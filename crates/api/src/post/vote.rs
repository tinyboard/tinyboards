use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{CreatePostVote, PostIdPath},
    site::Message,
    utils::{
        blocking, check_board_deleted_or_removed, check_post_deleted_removed_or_locked,
        require_user,
    },
};
use tinyboards_db::{
    models::post::post_vote::{PostVote, PostVoteForm},
    models::{board::board::Board, post::post::Post},
    traits::{Crud, Voteable},
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreatePostVote {
    type Response = Message;
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

        let post = blocking(context.pool(), move |conn| {
            Post::read(conn, post_id).map_err(|_| TinyBoardsError::err_500())
        })
        .await??;

        // check if post can be liked (not deleted, removed, or locked)
        check_post_deleted_removed_or_locked(post_id, context.pool()).await?;

        // check if the board is there or not
        check_board_deleted_or_removed(post.board_id, context.pool()).await?;

        // check if user is banned from board
        let is_board_banned = blocking(context.pool(), move |conn| {
            Board::board_has_ban(conn, post.board_id, user.id)
        })
        .await?
        .map_err(|_| TinyBoardsError::err_500())?;

        if !is_board_banned {
            let vote_form = PostVoteForm {
                post_id: path.post_id,
                user_id: user.id,
                score: data.score,
            };

            // remove any existing votes first
            let user_id = user.id;
            blocking(context.pool(), move |conn| {
                PostVote::remove(conn, user_id, post_id)
            })
            .await??;

            let do_add = vote_form.score != 0 && (vote_form.score == 1 || vote_form.score == -1);

            if do_add {
                let cloned_form = vote_form.clone();
                let like = move |conn: &mut _| PostVote::vote(conn, &cloned_form);
                blocking(context.pool(), like)
                    .await?
                    .map_err(|_e| TinyBoardsError::from_string("could not vote on post", 500))?;
            } else {
                let cloned_form = vote_form.clone();
                let like = move |conn: &mut _| {
                    PostVote::remove(conn, cloned_form.user_id, cloned_form.post_id)
                };
                blocking(context.pool(), like).await?.map_err(|_e| {
                    TinyBoardsError::from_string("could not remove vote on post", 500)
                })?;
            }
        }

        // mark the post as read here

        // grab the post view here for the response
        /*let post_view = blocking(context.pool(), move |conn| {
            PostView::read(conn, vote_form.post_id, Some(vote_form.user_id))
                .map_err(|_e| TinyBoardsError::from_string("could not find post", 404))
        })
        .await??;*/

        Ok(Message::new("Vote saved!"))
    }
}

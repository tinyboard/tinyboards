use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    post::{CreatePostVote, PostResponse, PostIdPath},
    utils::{
        blocking,
        get_user_view_from_jwt,
        check_board_ban,
        check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked,
        check_user_valid,
    }, 
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::post::post::Post,
    models::post::post_vote::{PostVote, PostVoteForm},
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
        let user_view = 
            get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;

        // check if downvotes are disabled (when/if we implement this feature)

        let post_id = path.post_id;

        let post 
            = blocking(context.pool(), move |conn| {
                Post::read(conn, post_id)
                .map_err(|_| TinyBoardsError::err_500())
            })
            .await??;
        
        // check if post can be liked (not deleted, removed, or locked)
        check_post_deleted_removed_or_locked(
            post_id, 
            context.pool()
        )
        .await?;

        // check if the user is banned from the board
        check_board_ban(
            user_view.user.id, 
            post.board_id, 
            context.pool()
        )
        .await?;

        // check if the board is there or not
        check_board_deleted_or_removed(
            post.board_id, 
            context.pool(),
        )
        .await?;

        // check if the user is valid
        check_user_valid(
            user_view.user.banned, 
            user_view.user.expires, 
            user_view.user.deleted
        )
        ?;

        let vote_form = PostVoteForm {
            post_id: path.post_id,
            user_id: user_view.user.id,
            score: data.score,
        };

        // remove any existing votes first
        let user_id = user_view.user.id;
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
            let like = move |conn: &mut _| PostVote::remove(conn, cloned_form.user_id, cloned_form.post_id);
            blocking(context.pool(), like)
                .await?
                .map_err(|_e| TinyBoardsError::from_string("could not remove vote on post", 500))?;
        }

        // mark the post as read here

        // grab the post view here for the response
        let post_view =
            blocking(context.pool(), move |conn| {
                PostView::read(conn, vote_form.post_id, Some(vote_form.user_id))
                    .map_err(|_e| TinyBoardsError::from_string("could not find post", 404))
            })
            .await??;

        Ok(PostResponse { post_view })
    }
}
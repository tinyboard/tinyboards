use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    post::{CreatePostLike, PostMessageResponse},
    utils::{
        blocking,
        get_user_view_from_jwt,
        check_board_ban,
        check_board_deleted_or_removed,
        check_post_deleted_removed_or_locked,
        check_user_valid,
    }, 
    data::PorplContext,
};
use porpl_db::{
    models::post::post::Post,
    models::post::post_like::{PostLike, PostLikeForm},
    traits::{Crud, Likeable},
};
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CreatePostLike {
    type Response = PostMessageResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {

        let data: &CreatePostLike = &self;
        let user_view = 
            get_user_view_from_jwt(&data.auth, context.pool(), context.master_key()).await?;

        // check if downvotes are disabled (when/if we implement this feature)

        let post_id = data.post_id;

        let post 
            = blocking(context.pool(), move |conn| {
                Post::read(conn, post_id)
                .map_err(|_| PorplError::err_500())
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

        let like_form = PostLikeForm {
            post_id: data.post_id,
            user_id: user_view.user.id,
            score: data.score,
        };

        // remove any existing votes first
        let user_id = user_view.user.id;
        blocking(context.pool(), move |conn| {
            PostLike::remove(conn, user_id, post_id)
        })
        .await??;

        let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);

        if do_add {
            let cloned_form = like_form.clone();
            let like = move |conn: &mut _| PostLike::vote(conn, &cloned_form);
            blocking(context.pool(), like)
                .await?
                .map_err(|_e| PorplError { message: String::from("could_not_vote_on_post"), error_code: 500})?;       
        } else {
            let cloned_form = like_form.clone();
            let like = move |conn: &mut _| PostLike::remove(conn, cloned_form.user_id, cloned_form.post_id);
            blocking(context.pool(), like)
                .await?
                .map_err(|_e| PorplError { message: String::from("could_not_remove_vote_on_post"), error_code: 500 })?;
        }

        // mark the post as read here

        Ok(PostMessageResponse {
            status_code: 200,
            message: String::from("post liked successfully"),
        })
    }
}
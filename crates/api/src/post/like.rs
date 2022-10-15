use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    post::{CreatePostLike, PostMessageResponse},
    utils::{
        blocking,
        get_user_view_from_jwt,
    }, 
    data::PorplContext,
};
use porpl_db::{
    //models::post::post::Post,
    models::post::post_like::{PostLike, PostLikeForm},
    traits::{/*Crud,*/ Likeable},
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
        
        // below will be used when checking if user is banned from board, or if board was removed or deleted.
        
        // let post: Post = blocking(context.pool(), move |conn| { 
        //     Post::read(conn, post_id)
        //     .map_err(|e| {
        //         eprintln!("ERROR: {}", e);
        //         PorplError::err_500()
        //     })
        // })
        // .await??
        // .into();

        // check to see is user is banned from board here (can't vote if so)
        // check to see if board has been removed or deleted here (can't vote if so)

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
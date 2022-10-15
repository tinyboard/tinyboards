use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    post::{DeletePost, PostMessageResponse},
    utils::{
        blocking,
        get_user_view_from_jwt,
        check_board_deleted_or_removed,
        check_board_ban,
    },
    data::PorplContext,
};
use porpl_db::{
    models::post::post::{Post},
    traits::Crud,
};
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeletePost {
    type Response = PostMessageResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<PostMessageResponse, PorplError> {
        let data: &DeletePost = &self;
        let user_view
             = get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key()).await?;

        let post_id = data.post_id;
        let orig_post = blocking(context.pool(), move |conn| {
                Post::read(conn, post_id)
                    .map_err(|e| {
                        eprintln!("ERROR: {}", e);
                        PorplError::err_500()
                    })
            }).await??;
        
        if orig_post.deleted == data.deleted {
            return Err(PorplError::from_string("couldn't delete post again", 500));
        }

        check_board_ban(
            user_view.user.id, 
            orig_post.board_id, 
            context.pool()
        )
        .await?;

        check_board_deleted_or_removed(
            orig_post.board_id, 
            context.pool()
        )
        .await?;

        if !Post::is_post_creator(user_view.user.id, orig_post.creator_id) {
            return Err(PorplError::from_string("post edit not allowed", 401));
        }

        let post_id = data.post_id;
        let deleted = data.deleted;
        let _updated_post = blocking(context.pool(), move |conn| {
            Post::update_deleted(conn, post_id, deleted)
                .map_err(|_e| PorplError::err_500())
        })
        .await??;

        Ok(PostMessageResponse{
            message: String::from("post deleted successfully"),
            status_code: 200,
        })
    }
}
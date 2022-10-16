use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{SubmitPost, SubmitPostResponse},
    utils::{blocking, 
            get_user_view_from_jwt,
            check_board_deleted_or_removed,
            check_board_ban,
            check_user_valid,
        },
};
use porpl_db::models::{
    post::post::{Post, PostForm},
};
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for SubmitPost {
    type Response = SubmitPostResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<SubmitPostResponse, PorplError> {
        let data: SubmitPost = self;

        let user_view = get_user_view_from_jwt(auth.unwrap(), context.pool(), context.master_key())
            .await?;

        // check to see if user is banned from board        
        check_board_ban(
            user_view.user.id,
            data.board_id, 
            context.pool()
        )
        .await?;

        // check to see if board is removed or deleted
        check_board_deleted_or_removed(
            data.board_id, 
            context.pool()
        )
        .await?;

        // check to see if user is valid (not banned or deleted)
        check_user_valid(
            user_view.user.banned, 
            user_view.user.expires, 
            user_view.user.deleted,
        )?;

        let post_form = PostForm {
            title: data.title,
            type_: data.type_,
            url: data.url,
            body: data.body,
            creator_id: user_view.user.id,
            board_id: data.board_id,
            nsfw: Some(data.nsfw),
            ..PostForm::default()
        };

        let _published_post =
            blocking(context.pool(), move |conn| Post::submit(conn, post_form)).await??;

        let submit_post_response = SubmitPostResponse {
            message: String::from("Post submitted successfully!"),
            status_code: 200,
        };

        Ok(submit_post_response)
    }
}
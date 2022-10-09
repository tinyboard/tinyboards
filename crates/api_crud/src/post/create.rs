use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{SubmitPost, SubmitPostResponse},
    utils::blocking,
};
use porpl_db::{
    models::{
        post::post::{Post, PostForm},
        user::user::User,
    }
};
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for SubmitPost {
    type Response = SubmitPostResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Option<&str>,
    ) -> Result<SubmitPostResponse, PorplError> {
        let data: SubmitPost = self;

        // some sanitization logic here
        //let logged_in: User = User::from_jwt(conn, token, context.master_key());

        let post_form = PostForm {
            title: Some(data.title),
            type_: data.type_,
            url: Some(data.url),
            body: Some(data.body),
            creator_id: Some(data.creator_id),
            board_id: Some(data.board_id),
            nsfw: Some(data.nsfw),
            ..PostForm::default()
        };

        let published_post =
            blocking(context.pool(), move |conn| {
                Post::submit(conn, post_form)
            })
            .await??;

        let submit_post_response = SubmitPostResponse {
            message: String::from("Post submitted successfully!"),
            status_code: 200,
        };

        Ok(submit_post_response)
    }
}
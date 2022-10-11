use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{SubmitPost, SubmitPostResponse},
    utils::{blocking, require_user},
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

        let u = require_user(context.pool(), context.master_key(), auth).await?;

        let post_form = PostForm {
            title: data.title,
            type_: data.type_,
            url: data.url,
            body: data.body,
            creator_id: u.id,
            board_id: data.board_id,
            nsfw: Some(data.nsfw),
            ..PostForm::default()
        };

        let published_post =
            blocking(context.pool(), move |conn| Post::submit(conn, post_form)).await??;

        let submit_post_response = SubmitPostResponse {
            message: String::from("Post submitted successfully!"),
            status_code: 200,
        };

        Ok(submit_post_response)
    }
}
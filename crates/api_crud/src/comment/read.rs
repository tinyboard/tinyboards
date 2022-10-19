use crate::PerformCrud;
use actix_web::web;
use porpl_api_common::{
    comment::{GetPostComments, GetPostCommentsRoute},
    data::PorplContext,
    utils::blocking,
};
use porpl_db::models::{comment::comment::Comment, post::post::Post};
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPostComments {
    type Response = Vec<Comment>;
    type Route = GetPostCommentsRoute;

    async fn perform(
        self,
        context: &web::Data<PorplContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        // check if post exists
        if blocking(context.pool(), move |conn| {
            Post::check_if_exists(conn, path.post_id)
        })
        .await??
        .is_none()
        {
            return Err(PorplError::from_string("Invalid post ID", 404));
        }

        blocking(context.pool(), move |conn| {
            Comment::replies_to_post(conn, path.post_id)
        })
        .await?
    }
}

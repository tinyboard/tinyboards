use crate::PerformCrud;
use actix_web::web;
use porpl_api_common::{
    comment::{GetPostComments, GetPostCommentsRoute},
    data::PorplContext,
    utils::{blocking, load_user_opt},
};
use porpl_db::models::post::post::Post;
use porpl_db_views::{comment_view::CommentQuery, structs::CommentView};
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPostComments {
    type Response = Vec<CommentView>;
    type Route = GetPostCommentsRoute;

    async fn perform(
        self,
        context: &web::Data<PorplContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;
        // check if post exists
        if blocking(context.pool(), move |conn| {
            Post::check_if_exists(conn, path.post_id)
        })
        .await??
        .is_none()
        {
            return Err(PorplError::from_string("Invalid post ID", 404));
        }

        let comments = blocking(context.pool(), move |conn| {
            CommentQuery::builder()
                .conn(conn)
                //.sort(None)
                .post_id(Some(path.post_id))
                .show_deleted_and_removed(Some(true))
                .user(user.as_ref())
                //.page(None)
                //.limit(None)
                .build()
                .list()
        })
        .await?
        .map_err(|_| PorplError::err_500())?;

        let comments = CommentView::into_tree(comments);

        Ok(comments)
    }
}

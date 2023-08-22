use crate::PerformCrud;
use actix_web::web;
use tinyboards_api_common::{
    comment::{CommentIdPath, GetComment, CommentResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt}, build_response::build_comment_response,
};
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &web::Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let _data = self;

        let view = load_user_opt(context.pool(), context.master_key(), auth)
            .await?;

        // check if the instance is private before listing comments
        check_private_instance(&view, context.pool()).await?;

        //let person_id = user.as_ref().map(|u| u.id);
        let comment_id = path.comment_id;

        // let comment_context = data.context;
        // let sort = match data.sort.as_ref() {
        //     Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
        //     None => CommentSortType::Hot,
        // };

        Ok(build_comment_response(context, comment_id, view, vec![]).await?)
    }
}

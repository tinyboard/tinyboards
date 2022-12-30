use crate::PerformCrud;
use actix_web::web;
use tinyboards_api_common::{
    comment::{CommentIdPath, GetComment, ListCommentsResponse},
    data::TinyBoardsContext,
    utils::{blocking, check_private_instance, load_user_opt},
};
use tinyboards_db::{map_to_comment_sort_type, CommentSortType};
use tinyboards_db_views::structs::CommentView;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetComment {
    type Response = ListCommentsResponse;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &web::Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data = self;

        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if the instance is private before listing comments
        check_private_instance(&user, context.pool()).await?;

        //let user_id = user.as_ref().map(|u| u.id);
        let comment_id = path.comment_id;

        let comment_context = data.context;
        let sort = match data.sort.as_ref() {
            Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
            None => CommentSortType::Hot,
        };

        let comment_query_response = blocking(context.pool(), move |conn| {
            CommentView::get_comment_with_replies(
                conn,
                comment_id,
                Some(sort),
                user.as_ref(),
                comment_context,
                data.post,
            )
        })
        .await??;

        Ok(ListCommentsResponse {
            comments: comment_query_response.comments,
            total_count: comment_query_response.count,
        })
    }
}

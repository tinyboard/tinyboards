use tinyboards_db::{map_to_comment_sort_type, CommentSortType};
use tinyboards_db_views::{structs::CommentView, comment_view::CommentQueryResponse};
use crate::PerformCrud;
use actix_web::web;
use tinyboards_api_common::{
    comment::{CommentIdPath, GetComment, ListCommentsResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt},
};
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

        let v = load_user_opt(context.pool(), context.master_key(), auth)
            .await?;

        // check if the instance is private before listing comments
        check_private_instance(&v, context.pool()).await?;

        let person = v.map(|v| v.person);
        let comment_id = path.comment_id;
        let post_id = data.post;

        let comment_context = data.context;
        let sort = match data.sort.as_ref() {
            Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
            None => CommentSortType::Hot,
        };

        let CommentQueryResponse { comments, count }
            = CommentView::get_comment_with_replies(
                context.pool(),
                comment_id,
                Some(sort),
                person.as_ref(),
                comment_context,
                post_id
            ).await?;

        Ok(ListCommentsResponse {
            comments,
            total_count: count
        })
    }
}

use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    comment::{GetComments, GetCommentsResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db_views::comment_view::CommentQuery;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetComments {
    type Response = GetCommentsResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetCommentsResponse, TinyBoardsError> {
        let data = self;
        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;
        
        // check to see if instance is set to private before listing comments
        check_private_instance(&v, context.pool()).await?;

        let person_id = v.as_ref().map(|u| u.person.id);
        
        let comments = CommentQuery::builder()
            .pool(context.pool())
            .listing_type(data.type_)
            .sort(None) // TODO: convert String to CommentSortType if needed
            .board_id(data.board_id)
            .post_id(data.post_id)
            .creator_id(data.creator_id)
            .person_id(person_id)
            .saved_only(data.saved_only)
            .page(data.page)
            .limit(data.limit)
            .build()
            .list()
            .await?;

        Ok(GetCommentsResponse { 
            comments: comments.comments,
            total_count: comments.count,
        })
    }
}
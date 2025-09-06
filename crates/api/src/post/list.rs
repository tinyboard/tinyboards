use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{GetPosts, GetPostsResponse},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db_views::post_view::PostQuery;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetPosts {
    type Response = GetPostsResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetPostsResponse, TinyBoardsError> {
        let data = self;
        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;
        
        // check to see if instance is set to private before listing posts
        check_private_instance(&v, context.pool()).await?;

        let person_id = v.as_ref().map(|u| u.person.id);
        
        let posts = PostQuery::builder()
            .pool(context.pool())
            .sort(None) // TODO: convert String to PostSortType if needed  
            .listing_type(data.type_)
            .board_id(data.board_id)
            .creator_id(data.creator_id)
            .saved_only(data.saved_only)
            .page(data.page)
            .limit(data.limit)
            .build()
            .list()
            .await?;

        Ok(GetPostsResponse { 
            posts: posts.posts,
            total_count: posts.count,
        })
    }
}
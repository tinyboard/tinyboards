use crate::PerformQuery;
use actix_web::web::{Data, Query};
use tinyboards_api_common::{
    site::GetFeed,
    post::ListPostsResponse,
    data::TinyBoardsContext,
};
use tinyboards_utils::error::TinyBoardsError;
use diesel::{result::Error, *};



#[async_trait::async_trait(?Send)]
impl<'des> PerformQuery<'des> for GetFeed {

    type Response = ListPostsResponse;
    type QueryForm = GetFeed;

    #[tracing::instrument(skip_all)]
    async fn perform_query(
        self,
        context: &Data<TinyBoardsContext>,
        params: Query<Self::QueryForm>,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        
        todo!()
    }

}

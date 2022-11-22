use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetMembers, GetMembersResponse},
    utils::{blocking, check_private_instance, get_user_view_from_jwt_opt},
};
use tinyboards_db_views::user_view::UserQuery;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetMembers {
    type Response = GetMembersResponse;
    type Route = ();

    #[tracing::instrument(skip_all)]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let params: &Self = &self;

        // get optional user view (don't need to be logged in)
        let user_view = 
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;

        // check if members should be shown or not
        check_private_instance(&user_view, context.pool()).await?;

        let sort = params.sort.clone();
        let limit = params.limit;
        let page = params.page;

        let members = blocking(context.pool(), move |conn| {
            UserQuery::builder()
                .conn(conn)
                .sort(sort)
                .limit(limit)
                .page(page)
                .build()
                .list()
        })
        .await??;

        Ok(GetMembersResponse { members })
    }
}
use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetMembers, GetMembersResponse},
    utils::{blocking, check_private_instance, load_user_opt},
};
use tinyboards_db_views::user_view::UserQuery;
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_db::{UserSortType, map_to_user_sort_type};

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
        let user = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if members should be shown or not
        check_private_instance(&user, context.pool()).await?;

        let sort = match params.sort.as_ref() {
            Some(sort) => map_to_user_sort_type(Some(&sort.to_lowercase())),
            None => UserSortType::MostRep,
        };

        let limit = params.limit;
        let page = params.page;
        let is_admin = params.is_admin;
        let is_banned = params.is_banned;

        let response = blocking(context.pool(), move |conn| {
            UserQuery::builder()
                .conn(conn)
                .sort(Some(sort))
                .is_admin(is_admin)
                .is_banned(is_banned)
                .limit(limit)
                .page(page)
                .build()
                .list()
        })
        .await??;

        let members = response.users;
        let total_count = response.count;

        Ok(GetMembersResponse { members, total_count })
    }
}

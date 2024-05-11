use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetMembers, GetMembersResponse},
    utils::{check_private_instance, load_user_opt},
};
use tinyboards_db::{map_to_user_sort_type, models::site::local_site::LocalSite, UserSortType};
use tinyboards_db_views::person_view::PersonQuery;
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

        // get optional local user (don't need to be logged in)
        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check if members should be shown or not
        check_private_instance(&v, context.pool()).await?;

        let sort = match params.sort.as_ref() {
            Some(sort) => map_to_user_sort_type(Some(&sort.to_lowercase())),
            None => UserSortType::MostRep,
        };

        let limit = params.limit;
        let page = params.page;
        let is_admin = params.is_admin;
        let is_banned = params.is_banned;
        let search_term = params.search_term.clone();

        let local_site = LocalSite::read(context.pool()).await?;

        if local_site.open_registration == false
            && local_site.invite_only == false
            && local_site.require_application == true
        {
            let response = PersonQuery::builder()
                .pool(context.pool())
                .sort(Some(sort))
                .is_admin(is_admin)
                .is_banned(is_banned)
                .search_term(search_term)
                .approved_only(Some(true)) // we only want to display approved users
                .limit(limit)
                .page(page)
                .build()
                .list()
                .await?;

            let members = response.persons;
            let total_count = response.count;

            Ok(GetMembersResponse {
                members,
                total_count,
            })
        } else {
            let response = PersonQuery::builder()
                .pool(context.pool())
                .sort(Some(sort))
                .is_admin(is_admin)
                .is_banned(is_banned)
                .search_term(search_term)
                .limit(limit)
                .page(page)
                .build()
                .list()
                .await?;

            let members = response.persons;
            let total_count = response.count;

            Ok(GetMembersResponse {
                members,
                total_count,
            })
        }
    }
}

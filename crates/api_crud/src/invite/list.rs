use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{ListSiteInvites, ListSiteInvitesResponse},
    utils::{require_user},
};
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_db_views::site_invite_view::InviteQuery;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListSiteInvites {
    type Response = ListSiteInvitesResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        let data: &ListSiteInvites = &self;
        let page = data.page.clone();
        let limit  = data.limit.clone();

        // only admins should be able to list invites
        let _user = require_user(context.pool(), context.master_key(), auth)
        .await
        .require_admin()
        .unwrap()?;

        let response = InviteQuery::builder()
            .pool(context.pool())
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await?;

        let invites = response.invites;
        let total_count = response.count;

        Ok(ListSiteInvitesResponse { invites, total_count })
    }
}
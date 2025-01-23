use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    applications::{ListRegistrationApplications, ListRegistrationApplicationsResponse},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::models::person::local_user::AdminPerms;
use tinyboards_db_views::registration_application_view::ApplicationQuery;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListRegistrationApplications {
    type Response = ListRegistrationApplicationsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ListRegistrationApplications = &self;
        let page = data.page.clone();
        let limit = data.limit.clone();

        // only admins should be able to list site applications
        let _user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Users)
            .unwrap()?;

        let response = ApplicationQuery::builder()
            .pool(context.pool())
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await?;

        let applications = response.applications;
        let total_count = response.count;

        Ok(ListRegistrationApplicationsResponse {
            applications,
            total_count,
        })
    }
}

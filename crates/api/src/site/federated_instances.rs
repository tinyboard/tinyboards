use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetFederatedInstances, GetFederatedInstancesResponse},
    utils::build_federated_instances,
};
use tinyboards_db_views::structs::SiteView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetFederatedInstances {
    type Response = GetFederatedInstancesResponse;
    type Route = ();
    
    #[tracing::instrument(skip_all)]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _path: Self::Route,
        _auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let site_view = SiteView::read_local(context.pool()).await?;
        let federated_instances = 
            build_federated_instances(&site_view.local_site, context.pool()).await?;
        Ok(Self::Response {
            federated_instances
        })
    }
}
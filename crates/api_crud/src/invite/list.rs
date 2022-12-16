use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{ListSiteInvites},
    utils::{require_user, blocking},
};
use tinyboards_db::{
    models::{
        site::site_invite::{SiteInvite},
    }, 
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListSiteInvites {
    type Response = Vec<SiteInvite>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        // only admins should be able to list invites
        let _user = require_user(context.pool(), context.master_key(), auth)
        .await
        .require_admin()
        .unwrap()?;

        // get a list of site invites
        let invites = blocking(context.pool(), move  |conn| {
            SiteInvite::list(conn)
        })
        .await??;
        
        Ok(invites)
    }
}
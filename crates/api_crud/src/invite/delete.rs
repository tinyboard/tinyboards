use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{DeleteSiteInvite},
    utils::{require_user, blocking},
};
use tinyboards_db::{
    models::{
        site::site_invite::{SiteInvite},
    }, traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteSiteInvite {
    type Response = ();
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        let data: &DeleteSiteInvite = &self;
        let id = data.invite_id;

        // only admins should be able to delete invites
        let _user = require_user(context.pool(), context.master_key(), auth)
        .await
        .require_admin()
        .unwrap()?;

        // delete site invite
        blocking(context.pool(), move  |conn| {
            SiteInvite::delete(conn, id)
        })
        .await??;

        Ok(())
    }
}
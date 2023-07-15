use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{DeleteSiteInvite, InviteId},
    utils::require_user,
};
use tinyboards_db::{
    models::site::site_invite::SiteInvite, 
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteSiteInvite {
    type Response = ();
    type Route = InviteId;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        let id = path.invite_id.clone();

        // only admins should be able to delete invites
        require_user(context.pool(), context.master_key(), auth)
        .await
        .require_admin()
        .unwrap()?;

        // delete site invite
        SiteInvite::delete(context.pool(), id).await?;

        Ok(())
    }
}
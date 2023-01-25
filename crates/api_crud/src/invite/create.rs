use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{CreateSiteInvite, CreateSiteInviteResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::{
        site::site::Site,
        site::site_invite::{SiteInvite, SiteInviteForm},
    },
    traits::Crud,
};
use tinyboards_utils::{error::TinyBoardsError, utils::generate_rand_string};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateSiteInvite {
    type Response = CreateSiteInviteResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        // only admins should be able to create invites
        let _user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        // we only create invites if site is in invite mode
        let site = blocking(context.pool(), move |conn| Site::read_local(conn)).await??;

        if !site.invite_only {
            return Err(TinyBoardsError::from_message(
                400,
                "can't create an invite outside of invite mode",
            ));
        }

        let form = SiteInviteForm {
            verification_code: generate_rand_string(),
        };

        // create record in db
        let invite =
            blocking(context.pool(), move |conn| SiteInvite::create(conn, &form)).await??;

        let invite_url = format!(
            "{}/validate_invite/{}",
            context.settings().get_protocol_and_hostname(),
            invite.verification_code,
        );

        Ok(CreateSiteInviteResponse { invite_url })
    }
}

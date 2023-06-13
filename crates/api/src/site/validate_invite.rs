use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{InviteToken, ValidateSiteInvite},
};
use tinyboards_db::models::{site::{site_invite::SiteInvite}, site::local_site::LocalSite};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ValidateSiteInvite {
    type Response = ();
    type Route = InviteToken;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let token = path.invite_token.clone();

        let site = LocalSite::read(context.pool()).await?;

        if !site.invite_only {
            return Err(TinyBoardsError::from_message(
                400,
                "site is not in invite only mode",
            ));
        }

        let invite = SiteInvite::read_for_token(context.pool(), &token.as_str()).await?;

        if path.invite_token.clone() == invite.verification_code {
            Ok(())
        } else {
            Err(TinyBoardsError::from_message(
                500,
                "invite validation failed",
            ))
        }
    }
}

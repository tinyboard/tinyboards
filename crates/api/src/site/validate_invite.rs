use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    utils::{blocking},
    site::{ValidateSiteInvite},
};
use tinyboards_db::models::site::{site_invite::SiteInvite, site::Site};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ValidateSiteInvite {
    type Response = ();
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<(), TinyBoardsError> {

        let token = self.invite_token.clone();

        let site = blocking(context.pool(), move |conn| {
            Site::read_local(conn)
        })
        .await??;

        if !site.invite_only {
            return Err(TinyBoardsError::from_message("site is not in invite only mode"));
        }

        let invite = blocking(context.pool(), move |conn| {
            SiteInvite::read_for_token(conn, &token.as_str())
        })
        .await??;

        if self.invite_token == invite.verification_code {
            Ok(())
        } else {
            Err(TinyBoardsError::from_message("invite validation failed"))
        }
    }
}
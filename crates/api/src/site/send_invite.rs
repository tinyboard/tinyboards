use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{SendInvite, Message},
    utils::{blocking, require_user},
};
use tinyboards_db::models::site::site_invite::{SiteInvite, SiteInviteForm};
use tinyboards_utils::{
    error::TinyBoardsError,
    email::send_email,
    utils::is_valid_email,
};
use uuid::Uuid;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SendInvite {
    type Response = Message;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Message, TinyBoardsError> {

        let data: &SendInvite = &self;

        let email = data.email.clone();

        // validate that the email is in the right format or not
        is_valid_email(&email.as_str())?;

        // require an admin to create invite
        require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        // create a UUIDv4 for the invite token
        let token = Uuid::new_v4().to_string();

        let form = SiteInviteForm {
            email,
            token
        };




        todo!()
    }

}
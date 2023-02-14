use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{BanUser, ModActionResponse},
    utils::require_user,
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModBan, ModBanForm},
    models::user::users::User,
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BanUser {
    type Response = ModActionResponse<ModBan>;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &BanUser = &self;
        let target_user_id = data.target_user_id;
        let banned = data.banned;
        let reason = &data.reason;
        let expires = data.expires;

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        // update the user in the database to be banned
        User::update_ban(context.pool(), target_user_id.clone(), banned.clone()).await?;

        // form for submitting ban action for mod log
        let ban_form = ModBanForm {
            mod_user_id: user.id,
            other_user_id: target_user_id.clone(),
            banned: Some(Some(banned)),
            expires: Some(expires),
            reason: Some(reason.clone()),
        };

        // enter mod log action
        let mod_action = ModBan::create(context.pool(), &ban_form).await?;

        Ok(ModActionResponse { mod_action })
    }
}

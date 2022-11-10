use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{BanUser, ModActionResponse},
    utils::{blocking, get_user_view_from_jwt, is_admin},
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModBan, ModBanForm},
    models::user::user::User,
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
        let other_user_id = data.other_user_id;
        let banned = data.banned;
        let reason = &data.reason;
        let expires = data.expires;

        let user_view = get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        // first of all this user MUST be an admin to ban site-wide
        is_admin(context.pool(), user_view.user.id).await?;

        // update the user in the database to be banned
        blocking(context.pool(), move |conn| {
            User::update_ban(conn, other_user_id.clone(), banned.clone())
                .map_err(|_e| TinyBoardsError::from_string("could not ban user", 500))
        })
        .await??;

        // form for submitting ban action for mod log
        let ban_form = ModBanForm {
            mod_user_id: user_view.user.id,
            other_user_id: other_user_id.clone(),
            banned: Some(Some(banned)),
            expires: expires,
            reason: Some(Some(reason.clone())),
        };

        // enter mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            ModBan::create(conn, &ban_form)
                .map_err(|_e| TinyBoardsError::from_string("could not log mod action", 500))
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}

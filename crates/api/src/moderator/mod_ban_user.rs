use crate::Perform;
use crate::utils::naive_now;
use tinyboards_db::{
    models::moderator::mod_actions::{ModBan, ModBanForm},
    models::user::user::User,
    traits::Crud,
};
use actix_web::web::Data;
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_api_common::{
    moderator::{BanUser, ModActionResponse},
    utils::{
        blocking,
        get_user_view_from_jwt,
        is_mod_or_admin,
    },
    data::TinyBoardsContext,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BanUser {
    type Response = ModActionResponse<ModBanForm>;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &BanUser = &self;
        let mod_user_id = data.mod_user_id;
        let other_user_id = data.ban_user_id;
        let banned = data.banned;
        let reason = data.reason;
        let expires = data.expires;

        let user_view =
            get_user_view_from_jwt(auth.unwrap_or(""), context.pool(), context.master_key()).await?;

        // get the user object
        let orig_user = blocking(context.pool(), move |conn| {
            User::read(conn, user_id.clone())
                .map_err(|_e| TinyBoardsError::from_string("couldn't find user", 404))
        })
            .await??;

        // first of all this user MUST be an admin or a mod
        is_mod_or_admin(
            context.pool(),
            user_view.user.id,
            1,
        ).await?;

        // update the user in the database to be banned
        blocking(context.pool(), move |conn| {
            User::update_ban(conn, user_id.clone(), banned.clone())
                .map_err(|_e| TinyBoardsError::from_string("could not ban user", 500))
        })
            .await??;

        // form for submitting ban action for mod log
        let ban_form = ModBan {
            mod_user_id: user_view.user.id,
            other_user_id: orig_user.id,
            banned: banned,
            expires: expires.equals(naive_now()),
            reason: Some(reason),
            when_: expires.equals(naive_now())

        };

        // enter mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            ModBanForm::create(conn, &ban_form)
                .map_err(|_e| TinyBoardsError::from_string("could not log mod action", 500))
        })
            .await??;

        Ok( ModActionResponse { mod_action })
    }
}
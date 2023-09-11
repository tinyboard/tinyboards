use crate::Perform;
use actix_web::web::Data;
use chrono::NaiveDateTime;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{ModActionResponse, ToggleBan},
    utils::require_user,
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModBan, ModBanForm},
    models::person::person::Person,
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ToggleBan {
    type Response = ModActionResponse<ModBan>;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &ToggleBan = &self;
        let target_person_id = data.target_person_id;
        let banned = data.banned;
        let reason = &data.reason;
        let expires = data.expires;

        let expires = expires.map(|ts| NaiveDateTime::from_timestamp_opt(ts, 0).unwrap());

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        // update the person in the database to be banned/unbanned
        Person::update_ban(
            context.pool(),
            target_person_id.clone(),
            banned.clone(),
            expires.clone(),
        )
        .await?;

        // form for submitting ban action for mod log
        let ban_form = ModBanForm {
            mod_person_id: view.person.id,
            other_person_id: target_person_id.clone(),
            banned: Some(Some(banned)),
            expires: Some(expires),
            reason: Some(reason.clone()),
        };

        // enter mod log action
        let mod_action = ModBan::create(context.pool(), &ban_form).await?;

        Ok(ModActionResponse { mod_action })
    }
}

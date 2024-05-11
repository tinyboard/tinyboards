use crate::Perform;
use actix_web::web::Data;
use chrono::NaiveDateTime;
use std::fmt::Write;
use std::time::SystemTime;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{ModActionResponse, ToggleBan},
    utils::{require_user, send_system_message},
};
use tinyboards_db::{
    models::{
        moderator::mod_actions::{ModBan, ModBanForm},
        person::{local_user::AdminPerms, person::Person},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::LocalUserView;
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
        let target_name = &data.username;
        let banned = data.banned;
        let reason = &data.reason;
        let duration_days = data.duration_days;

        // timestamp of the date when the ban expires
        let expires = duration_days.clone().map(|days| {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|n| n.as_secs())
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "Invalid expiry timestamp")
                })
                .unwrap();

            let expiry_timestamp = now + (days * 60 * 60 * 24) as u64;

            NaiveDateTime::from_timestamp_opt(expiry_timestamp as i64, 0)
                .expect("Invalid value for `duration_days`")
        });

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Users)
            .unwrap()?;

        let target_user_view = LocalUserView::get_by_name(context.pool(), target_name).await?;
        let target_person_id = target_user_view.person.id;

        // update the person in the database to be banned/unbanned
        Person::update_ban(
            context.pool(),
            target_person_id.clone(),
            banned.clone(),
            expires.clone(),
        )
        .await?;

        // send ban/unban notif
        let message = if banned {
            let mut text = String::from("Your account has been ");

            match duration_days {
                Some(days) => {
                    write!(&mut text, "suspended for {} day(s) for ", days)
                        .expect("...there is no way this went wrong.");
                }
                None => text.push_str("permanently banned for "),
            }

            text.push_str(match reason {
                Some(reason) => reason,
                None => "breaking the rules",
            });

            text.push_str(".");

            text
        } else {
            "Your account has been unbanned! 🎉".into()
        };

        send_system_message(context.pool(), Some(target_person_id), None, message).await?;

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

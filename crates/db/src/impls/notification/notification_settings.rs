use crate::{
    models::notification::notification_settings::{NotificationSettings, NotificationSettingsForm},
    schema::notification_settings,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl NotificationSettings {
    pub async fn create(pool: &DbPool, form: &NotificationSettingsForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(notification_settings::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn create_default_for_user(pool: &DbPool, user_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        let form = NotificationSettingsForm {
            user_id,
            email_enabled: Some(true),
            comment_replies_enabled: Some(true),
            post_replies_enabled: Some(true),
            mentions_enabled: Some(true),
            post_votes_enabled: Some(false), // votes can be noisy by default
            comment_votes_enabled: Some(false),
            private_messages_enabled: Some(true),
            board_invites_enabled: Some(true),
            moderator_actions_enabled: Some(true),
            system_notifications_enabled: Some(true),
            created: Some(chrono::Utc::now().naive_utc()),
            updated: None,
        };

        diesel::insert_into(notification_settings::table)
            .values(&form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read(pool: &DbPool, user_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        notification_settings::table
            .filter(notification_settings::user_id.eq(user_id))
            .first::<Self>(conn)
            .await
    }

    pub async fn read_or_create_default(pool: &DbPool, user_id: i32) -> Result<Self, Error> {
        match Self::read(pool, user_id).await {
            Ok(settings) => Ok(settings),
            Err(Error::NotFound) => Self::create_default_for_user(pool, user_id).await,
            Err(e) => Err(e),
        }
    }

    pub async fn update(
        pool: &DbPool,
        user_id: i32,
        form: &NotificationSettingsForm,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let mut update_form = form.clone();
        update_form.updated = Some(Some(chrono::Utc::now().naive_utc()));

        diesel::update(notification_settings::table)
            .filter(notification_settings::user_id.eq(user_id))
            .set(&update_form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| match e {
                Error::NotFound => TinyBoardsError::from_message(404, "Notification settings not found"),
                _ => TinyBoardsError::from_error_message(e, 500, "Failed to update notification settings"),
            })
    }

    pub async fn delete(pool: &DbPool, user_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::delete(notification_settings::table)
            .filter(notification_settings::user_id.eq(user_id))
            .execute(conn)
            .await
    }

    pub async fn is_notification_enabled(
        pool: &DbPool,
        user_id: i32,
        notification_kind: &str,
    ) -> Result<bool, Error> {
        let settings = Self::read_or_create_default(pool, user_id).await?;

        let enabled = match notification_kind {
            "comment_reply" => settings.comment_replies_enabled,
            "post_reply" => settings.post_replies_enabled,
            "mention" => settings.mentions_enabled,
            "post_vote" => settings.post_votes_enabled,
            "comment_vote" => settings.comment_votes_enabled,
            "private_message" => settings.private_messages_enabled,
            "board_invite" => settings.board_invites_enabled,
            "moderator_action" => settings.moderator_actions_enabled,
            "system_notification" => settings.system_notifications_enabled,
            _ => false, // unknown types disabled by default
        };

        Ok(enabled)
    }

    pub async fn is_email_enabled(pool: &DbPool, user_id: i32) -> Result<bool, Error> {
        let settings = Self::read_or_create_default(pool, user_id).await?;
        Ok(settings.email_enabled)
    }
}
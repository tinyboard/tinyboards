use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::notification::{
        notification_settings::{
            NotificationSettings as DbNotificationSettings,
            NotificationSettingsInsertForm,
            NotificationSettingsUpdateForm,
        },
    },
    schema::{notification_settings, notifications},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct NotificationMutations;

#[derive(SimpleObject)]
pub struct MarkNotificationsReadResponse {
    pub success: bool,
    pub marked_count: i32,
}

#[derive(SimpleObject)]
pub struct DeleteNotificationResponse {
    pub success: bool,
}

#[derive(SimpleObject)]
pub struct UpdateNotificationSettingsResponse {
    pub success: bool,
    pub settings: NotificationSettingsOutput,
}

#[derive(SimpleObject)]
pub struct NotificationSettingsOutput {
    pub email_enabled: bool,
    pub comment_replies_enabled: bool,
    pub post_replies_enabled: bool,
    pub mentions_enabled: bool,
    pub private_messages_enabled: bool,
    pub board_invites_enabled: bool,
    pub moderator_actions_enabled: bool,
    pub system_notifications_enabled: bool,
}

#[derive(InputObject)]
pub struct UpdateNotificationSettingsInput {
    pub email_enabled: Option<bool>,
    pub comment_replies_enabled: Option<bool>,
    pub post_replies_enabled: Option<bool>,
    pub mentions_enabled: Option<bool>,
    pub private_messages_enabled: Option<bool>,
    pub board_invites_enabled: Option<bool>,
    pub moderator_actions_enabled: Option<bool>,
    pub system_notifications_enabled: Option<bool>,
}

#[Object]
impl NotificationMutations {
    /// Mark specific notifications as read
    pub async fn mark_notifications_read(
        &self,
        ctx: &Context<'_>,
        notification_ids: Vec<ID>,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let uuids: Vec<Uuid> = notification_ids
            .iter()
            .map(|id| id.parse::<Uuid>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid notification ID"))?;

        let marked_count = diesel::update(
            notifications::table
                .filter(notifications::id.eq_any(&uuids))
                .filter(notifications::recipient_user_id.eq(user.id))
                .filter(notifications::is_read.eq(false))
        )
        .set(notifications::is_read.eq(true))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))? as i32;

        Ok(MarkNotificationsReadResponse {
            success: true,
            marked_count,
        })
    }

    /// Mark all notifications as read for the current user
    pub async fn mark_all_notifications_as_read(
        &self,
        ctx: &Context<'_>,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let marked_count = diesel::update(
            notifications::table
                .filter(notifications::recipient_user_id.eq(user.id))
                .filter(notifications::is_read.eq(false))
        )
        .set(notifications::is_read.eq(true))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))? as i32;

        Ok(MarkNotificationsReadResponse {
            success: true,
            marked_count,
        })
    }

    /// Delete a specific notification
    pub async fn delete_notification(
        &self,
        ctx: &Context<'_>,
        notification_id: ID,
    ) -> Result<DeleteNotificationResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let notif_id: Uuid = notification_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid notification ID".into()))?;

        let deleted_count = diesel::delete(
            notifications::table
                .filter(notifications::id.eq(notif_id))
                .filter(notifications::recipient_user_id.eq(user.id))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if deleted_count > 0 {
            Ok(DeleteNotificationResponse { success: true })
        } else {
            Err(TinyBoardsError::NotFound("Notification not found".into()).into())
        }
    }

    /// Update user's notification settings
    pub async fn update_notification_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateNotificationSettingsInput,
    ) -> Result<UpdateNotificationSettingsResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        // Check if settings exist
        let existing: Option<DbNotificationSettings> = notification_settings::table
            .filter(notification_settings::user_id.eq(user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let updated_settings = if let Some(_existing) = existing {
            // Update existing settings
            let form = NotificationSettingsUpdateForm {
                is_email_enabled: input.email_enabled,
                is_comment_replies_enabled: input.comment_replies_enabled,
                is_post_replies_enabled: input.post_replies_enabled,
                is_mentions_enabled: input.mentions_enabled,
                is_private_messages_enabled: input.private_messages_enabled,
                is_board_invites_enabled: input.board_invites_enabled,
                is_moderator_actions_enabled: input.moderator_actions_enabled,
                is_system_notifications_enabled: input.system_notifications_enabled,
                updated_at: Some(chrono::Utc::now()),
            };

            diesel::update(
                notification_settings::table
                    .filter(notification_settings::user_id.eq(user.id))
            )
            .set(&form)
            .get_result::<DbNotificationSettings>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        } else {
            // Create new settings with defaults + input overrides
            let form = NotificationSettingsInsertForm {
                user_id: user.id,
                is_email_enabled: input.email_enabled.unwrap_or(true),
                is_comment_replies_enabled: input.comment_replies_enabled.unwrap_or(true),
                is_post_replies_enabled: input.post_replies_enabled.unwrap_or(true),
                is_mentions_enabled: input.mentions_enabled.unwrap_or(true),
                is_private_messages_enabled: input.private_messages_enabled.unwrap_or(true),
                is_board_invites_enabled: input.board_invites_enabled.unwrap_or(true),
                is_moderator_actions_enabled: input.moderator_actions_enabled.unwrap_or(true),
                is_system_notifications_enabled: input.system_notifications_enabled.unwrap_or(true),
            };

            diesel::insert_into(notification_settings::table)
                .values(&form)
                .get_result::<DbNotificationSettings>(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?
        };

        Ok(UpdateNotificationSettingsResponse {
            success: true,
            settings: NotificationSettingsOutput {
                email_enabled: updated_settings.is_email_enabled,
                comment_replies_enabled: updated_settings.is_comment_replies_enabled,
                post_replies_enabled: updated_settings.is_post_replies_enabled,
                mentions_enabled: updated_settings.is_mentions_enabled,
                private_messages_enabled: updated_settings.is_private_messages_enabled,
                board_invites_enabled: updated_settings.is_board_invites_enabled,
                moderator_actions_enabled: updated_settings.is_moderator_actions_enabled,
                system_notifications_enabled: updated_settings.is_system_notifications_enabled,
            },
        })
    }
}

use async_graphql::*;
use tinyboards_db::{
    models::notification::{
        notifications::Notification as DbNotification,
        notification_settings::{NotificationSettings as DbNotificationSettings, NotificationSettingsForm},
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

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
    /// Mark a single notification as read
    pub async fn mark_notification_as_read(
        &self,
        ctx: &Context<'_>,
        notification_id: i32,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        match DbNotification::mark_as_read(pool, notification_id, user.id).await {
            Ok(_) => Ok(MarkNotificationsReadResponse {
                success: true,
                marked_count: 1,
            }),
            Err(e) => Err(e.into()),
        }
    }

    /// Mark all notifications as read for the current user
    pub async fn mark_all_notifications_as_read(
        &self,
        ctx: &Context<'_>,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let marked_count = DbNotification::mark_all_as_read(pool, user.id).await? as i32;

        Ok(MarkNotificationsReadResponse {
            success: true,
            marked_count,
        })
    }

    /// Mark specific notifications as read (legacy method for compatibility)
    pub async fn mark_notifications_read(
        &self,
        ctx: &Context<'_>,
        notification_ids: Option<Vec<i32>>,
        mark_all: Option<bool>,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let mark_all = mark_all.unwrap_or(false);

        let marked_count = if mark_all {
            DbNotification::mark_all_as_read(pool, user.id).await? as i32
        } else if let Some(ids) = notification_ids {
            if ids.is_empty() {
                return Ok(MarkNotificationsReadResponse {
                    success: true,
                    marked_count: 0,
                });
            }

            DbNotification::mark_many_as_read(pool, ids, user.id).await? as i32
        } else {
            return Err(TinyBoardsError::from_message(
                400,
                "Either notification_ids or mark_all must be provided",
            )
            .into());
        };

        Ok(MarkNotificationsReadResponse {
            success: true,
            marked_count,
        })
    }

    /// Delete a specific notification
    pub async fn delete_notification(
        &self,
        ctx: &Context<'_>,
        notification_id: i32,
    ) -> Result<DeleteNotificationResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        match DbNotification::delete(pool, notification_id, user.id).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    Ok(DeleteNotificationResponse { success: true })
                } else {
                    Err(TinyBoardsError::from_message(404, "Notification not found").into())
                }
            }
            Err(e) => Err(e.into()),
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

        let form = NotificationSettingsForm {
            user_id: user.id,
            email_enabled: input.email_enabled,
            comment_replies_enabled: input.comment_replies_enabled,
            post_replies_enabled: input.post_replies_enabled,
            mentions_enabled: input.mentions_enabled,
            private_messages_enabled: input.private_messages_enabled,
            board_invites_enabled: input.board_invites_enabled,
            moderator_actions_enabled: input.moderator_actions_enabled,
            system_notifications_enabled: input.system_notifications_enabled,
            created: None, // Don't update creation time
            updated: Some(Some(chrono::Utc::now().naive_utc())), // Will be set automatically in the implementation
        };

        match DbNotificationSettings::update(pool, user.id, &form).await {
            Ok(updated_settings) => Ok(UpdateNotificationSettingsResponse {
                success: true,
                settings: NotificationSettingsOutput::from(updated_settings),
            }),
            Err(e) => Err(e.into()),
        }
    }
}

impl From<DbNotificationSettings> for NotificationSettingsOutput {
    fn from(settings: DbNotificationSettings) -> Self {
        Self {
            email_enabled: settings.email_enabled,
            comment_replies_enabled: settings.comment_replies_enabled,
            post_replies_enabled: settings.post_replies_enabled,
            mentions_enabled: settings.mentions_enabled,
            private_messages_enabled: settings.private_messages_enabled,
            board_invites_enabled: settings.board_invites_enabled,
            moderator_actions_enabled: settings.moderator_actions_enabled,
            system_notifications_enabled: settings.system_notifications_enabled,
        }
    }
}
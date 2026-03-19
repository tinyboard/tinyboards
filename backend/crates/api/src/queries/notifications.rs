use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbNotificationKind,
    models::notification::{
        notification_settings::NotificationSettings as DbNotificationSettings,
        notifications::Notification as DbNotification,
    },
    schema::{notification_settings, notifications, private_messages},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct QueryNotifications;

#[derive(SimpleObject)]
pub struct Notification {
    pub id: ID,
    #[graphql(name = "type")]
    pub kind: String,
    #[graphql(name = "isRead")]
    pub is_read: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    pub comment_id: Option<ID>,
    pub post_id: Option<ID>,
    pub message_id: Option<ID>,
}

#[derive(SimpleObject)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub comment_replies_enabled: bool,
    pub post_replies_enabled: bool,
    pub mentions_enabled: bool,
    pub private_messages_enabled: bool,
    pub board_invites_enabled: bool,
    pub moderator_actions_enabled: bool,
    pub system_notifications_enabled: bool,
}

#[derive(SimpleObject)]
pub struct UnreadNotificationCount {
    pub total: i32,
    pub replies: i32,
    pub mentions: i32,
    pub private_messages: i32,
    pub activity: i32,
}

impl From<DbNotification> for Notification {
    fn from(n: DbNotification) -> Self {
        let kind_str = match n.kind {
            DbNotificationKind::CommentReply => "comment_reply",
            DbNotificationKind::PostReply => "post_reply",
            DbNotificationKind::Mention => "mention",
            DbNotificationKind::PrivateMessage => "private_message",
            DbNotificationKind::ModAction => "mod_action",
            DbNotificationKind::System => "system",
        };
        Self {
            id: n.id.to_string().into(),
            kind: kind_str.to_string(),
            is_read: n.is_read,
            created_at: n.created_at.to_string(),
            comment_id: n.comment_id.map(|id| id.to_string().into()),
            post_id: n.post_id.map(|id| id.to_string().into()),
            message_id: n.message_id.map(|id| id.to_string().into()),
        }
    }
}

#[Object]
impl QueryNotifications {
    /// Get user notifications with filtering
    pub async fn get_notifications(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "unreadOnly")] unread_only: Option<bool>,
        #[graphql(name = "kindFilter")] kind_filter: Option<String>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Notification>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(25).min(50).max(1);
        let offset = ((page - 1) * limit) as i64;
        let limit = limit as i64;

        let mut query = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .order(notifications::created_at.desc())
            .into_boxed();

        if unread_only.unwrap_or(false) {
            query = query.filter(notifications::is_read.eq(false));
        }

        // Parse kind filter to match enum variants
        if let Some(ref filter) = kind_filter {
            let kinds: Vec<DbNotificationKind> = match filter.as_str() {
                "replies" => vec![DbNotificationKind::CommentReply, DbNotificationKind::PostReply],
                "activity" => vec![DbNotificationKind::ModAction, DbNotificationKind::System],
                other => {
                    // Try parsing individual kind
                    let parsed: Vec<DbNotificationKind> = other
                        .split(',')
                        .filter_map(|k| match k.trim() {
                            "comment_reply" => Some(DbNotificationKind::CommentReply),
                            "post_reply" => Some(DbNotificationKind::PostReply),
                            "mention" => Some(DbNotificationKind::Mention),
                            "private_message" => Some(DbNotificationKind::PrivateMessage),
                            "mod_action" => Some(DbNotificationKind::ModAction),
                            "system" => Some(DbNotificationKind::System),
                            _ => None,
                        })
                        .collect();
                    parsed
                }
            };

            if !kinds.is_empty() {
                query = query.filter(notifications::kind.eq_any(kinds));
            }
        }

        let db_notifications: Vec<DbNotification> = query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(db_notifications.into_iter().map(Notification::from).collect())
    }

    /// Get user's notification settings
    pub async fn get_notification_settings(
        &self,
        ctx: &Context<'_>,
    ) -> Result<NotificationSettings> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let settings: Option<DbNotificationSettings> = notification_settings::table
            .filter(notification_settings::user_id.eq(user.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        match settings {
            Some(s) => Ok(NotificationSettings {
                email_enabled: s.is_email_enabled,
                comment_replies_enabled: s.is_comment_replies_enabled,
                post_replies_enabled: s.is_post_replies_enabled,
                mentions_enabled: s.is_mentions_enabled,
                private_messages_enabled: s.is_private_messages_enabled,
                board_invites_enabled: s.is_board_invites_enabled,
                moderator_actions_enabled: s.is_moderator_actions_enabled,
                system_notifications_enabled: s.is_system_notifications_enabled,
            }),
            // Return defaults if no settings row exists
            None => Ok(NotificationSettings {
                email_enabled: true,
                comment_replies_enabled: true,
                post_replies_enabled: true,
                mentions_enabled: true,
                private_messages_enabled: true,
                board_invites_enabled: true,
                moderator_actions_enabled: true,
                system_notifications_enabled: true,
            }),
        }
    }

    /// Get count of unread notifications by type
    pub async fn get_unread_notification_count(
        &self,
        ctx: &Context<'_>,
    ) -> Result<UnreadNotificationCount> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        // Count all unread notifications
        let total: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Count by kind
        let reply_kinds = vec![DbNotificationKind::CommentReply, DbNotificationKind::PostReply];
        let replies: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .filter(notifications::kind.eq_any(&reply_kinds))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let mentions: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .filter(notifications::kind.eq(DbNotificationKind::Mention))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Unread private messages from the messages table
        let pm_count: i64 = private_messages::table
            .filter(private_messages::recipient_id.eq(user.id))
            .filter(private_messages::is_read.eq(false))
            .filter(private_messages::deleted_at.is_null())
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let activity_kinds = vec![DbNotificationKind::ModAction, DbNotificationKind::System];
        let activity: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .filter(notifications::kind.eq_any(&activity_kinds))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(UnreadNotificationCount {
            total: total as i32,
            replies: replies as i32,
            mentions: mentions as i32,
            private_messages: pm_count as i32,
            activity: activity as i32,
        })
    }
}

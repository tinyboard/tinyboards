use async_graphql::*;
use tinyboards_db::{
    models::{
        post::posts::Post,
        comment::comments::Comment,
        notification::{
            notifications::Notification as DbNotification,
            notification_settings::{NotificationSettings as DbNotificationSettings, NotificationSettingsForm},
        },
    },
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::NaiveDateTime;
use std::collections::HashMap;


use crate::{
    structs::{comment::Comment as GqlComment, user::User as GqlPerson, post::Post as GqlPost},
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryNotifications;

#[derive(SimpleObject)]
pub struct Notification {
    pub id: i32,
    pub kind: String,
    pub is_read: bool,
    pub created: String,
    pub comment: Option<GqlComment>,
    pub post: Option<GqlPost>,
    pub person: Option<GqlPerson>,
}

#[derive(SimpleObject)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub comment_replies_enabled: bool,
    pub post_replies_enabled: bool,
    pub mentions_enabled: bool,
    pub post_votes_enabled: bool,
    pub comment_votes_enabled: bool,
    pub private_messages_enabled: bool,
    pub board_invites_enabled: bool,
    pub moderator_actions_enabled: bool,
    pub system_notifications_enabled: bool,
}

#[derive(SimpleObject)]
pub struct UnreadNotificationCount {
    pub total: i32,
    pub comment_replies: i32,
    pub post_replies: i32,
    pub mentions: i32,
    pub post_votes: i32,
    pub comment_votes: i32,
    pub private_messages: i32,
    pub board_invites: i32,
    pub moderator_actions: i32,
    pub system_notifications: i32,
}

#[derive(InputObject)]
pub struct NotificationFilters {
    pub unread_only: Option<bool>,
    pub kind_filter: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

#[Object]
impl QueryNotifications {
    /// Get user notifications with advanced filtering
    pub async fn get_notifications(
        &self,
        ctx: &Context<'_>,
        filters: Option<NotificationFilters>,
    ) -> Result<Vec<Notification>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let filters = filters.unwrap_or_default();
        let unread_only = filters.unread_only;
        let kind_filter = filters.kind_filter;
        let page = filters.page.unwrap_or(1).max(1);
        let limit = filters.limit.unwrap_or(25).min(50).max(1); // Cap at 50, min 1
        let offset = (page - 1) * limit;

        let db_notifications = DbNotification::get_for_user(
            pool,
            user.id,
            unread_only,
            kind_filter,
            Some(limit as i64),
            Some(offset as i64),
        )
        .await?;

        let mut result = Vec::new();
        for notification in db_notifications {
            // Load related comment/post/person if needed
            let comment = if let Some(comment_id) = notification.comment_id {
                match Comment::get_with_counts(pool, comment_id).await {
                    Ok(comment) => Some(GqlComment::from(comment)),
                    Err(_) => None,
                }
            } else {
                None
            };

            let post = if let Some(post_id) = notification.post_id {
                match Post::get_with_counts(pool, post_id, false).await {
                    Ok(post) => Some(GqlPost::from(post)),
                    Err(_) => None,
                }
            } else {
                None
            };

            // For now, we'll leave person as None since we don't have a direct person reference in notifications
            let person = None;

            result.push(Notification {
                id: notification.id,
                kind: notification.kind,
                is_read: notification.is_read,
                created: notification.created.to_string(),
                comment,
                post,
                person,
            });
        }

        Ok(result)
    }

    /// Get user's notification settings
    pub async fn get_notification_settings(
        &self,
        ctx: &Context<'_>,
    ) -> Result<NotificationSettings> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let settings = DbNotificationSettings::read_or_create_default(pool, user.id).await?;

        Ok(NotificationSettings {
            email_enabled: settings.email_enabled,
            comment_replies_enabled: settings.comment_replies_enabled,
            post_replies_enabled: settings.post_replies_enabled,
            mentions_enabled: settings.mentions_enabled,
            post_votes_enabled: settings.post_votes_enabled,
            comment_votes_enabled: settings.comment_votes_enabled,
            private_messages_enabled: settings.private_messages_enabled,
            board_invites_enabled: settings.board_invites_enabled,
            moderator_actions_enabled: settings.moderator_actions_enabled,
            system_notifications_enabled: settings.system_notifications_enabled,
        })
    }

    /// Get count of unread notifications by type
    pub async fn get_unread_notification_count(
        &self,
        ctx: &Context<'_>,
    ) -> Result<UnreadNotificationCount> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let total = DbNotification::count_unread_for_user(pool, user.id).await? as i32;
        let counts_by_kind = DbNotification::count_unread_by_kind(pool, user.id).await?;

        // Convert to HashMap for easy lookup
        let mut count_map: HashMap<String, i32> = HashMap::new();
        for (kind, count) in counts_by_kind {
            count_map.insert(kind, count as i32);
        }

        Ok(UnreadNotificationCount {
            total,
            comment_replies: count_map.get("comment_reply").copied().unwrap_or(0),
            post_replies: count_map.get("post_reply").copied().unwrap_or(0),
            mentions: count_map.get("mention").copied().unwrap_or(0),
            post_votes: count_map.get("post_vote").copied().unwrap_or(0),
            comment_votes: count_map.get("comment_vote").copied().unwrap_or(0),
            private_messages: count_map.get("private_message").copied().unwrap_or(0),
            board_invites: count_map.get("board_invite").copied().unwrap_or(0),
            moderator_actions: count_map.get("moderator_action").copied().unwrap_or(0),
            system_notifications: count_map.get("system_notification").copied().unwrap_or(0),
        })
    }
}

impl Default for NotificationFilters {
    fn default() -> Self {
        Self {
            unread_only: Some(false),
            kind_filter: None,
            page: Some(1),
            limit: Some(25),
        }
    }
}
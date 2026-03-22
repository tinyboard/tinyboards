use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbNotificationKind,
    models::notification::{
        notification_settings::NotificationSettings as DbNotificationSettings,
        notifications::Notification as DbNotification,
    },
    schema::{boards, comments, notification_settings, notifications, posts, private_messages, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct QueryNotifications;

/// Actor who triggered the notification
#[derive(SimpleObject, Clone)]
pub struct NotificationActor {
    pub id: ID,
    pub name: String,
    #[graphql(name = "displayName")]
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}

/// Context about the post related to the notification
#[derive(SimpleObject, Clone)]
pub struct NotificationPostContext {
    pub id: ID,
    pub title: String,
    #[graphql(name = "boardName")]
    pub board_name: String,
    #[graphql(name = "boardId")]
    pub board_id: ID,
}

/// Snippet from the comment related to the notification
#[derive(SimpleObject, Clone)]
pub struct NotificationCommentContext {
    pub id: ID,
    pub body: String,
    #[graphql(name = "postId")]
    pub post_id: ID,
    #[graphql(name = "postTitle")]
    pub post_title: String,
    #[graphql(name = "boardName")]
    pub board_name: String,
}

/// Context about a private message notification
#[derive(SimpleObject, Clone)]
pub struct NotificationMessageContext {
    pub id: ID,
    pub body: String,
}

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
    /// The user who triggered this notification
    pub actor: Option<NotificationActor>,
    /// Post context (title, board) if notification is post-related
    pub post: Option<NotificationPostContext>,
    /// Comment context (body snippet, post title) if notification is comment-related
    pub comment: Option<NotificationCommentContext>,
    /// Message context (body snippet) if notification is a private message
    pub message: Option<NotificationMessageContext>,
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

fn kind_to_str(kind: &DbNotificationKind) -> &'static str {
    match kind {
        DbNotificationKind::CommentReply => "comment_reply",
        DbNotificationKind::PostReply => "post_reply",
        DbNotificationKind::Mention => "mention",
        DbNotificationKind::PrivateMessage => "private_message",
        DbNotificationKind::ModAction => "mod_action",
        DbNotificationKind::System => "system",
    }
}

/// Truncate text to a snippet, breaking at word boundaries
fn truncate_snippet(text: &str, max_len: usize) -> String {
    let text = text.trim();
    if text.len() <= max_len {
        return text.to_string();
    }
    // Find the last space before max_len
    match text[..max_len].rfind(' ') {
        Some(pos) => format!("{}...", &text[..pos]),
        None => format!("{}...", &text[..max_len]),
    }
}

#[Object]
impl QueryNotifications {
    /// Get user notifications with filtering, enriched with actor/context data
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

        // Build the base query for notification rows
        let mut query = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .order(notifications::created_at.desc())
            .into_boxed();

        if unread_only.unwrap_or(false) {
            query = query.filter(notifications::is_read.eq(false));
        }

        if let Some(ref filter) = kind_filter {
            let kinds: Vec<DbNotificationKind> = match filter.as_str() {
                "replies" => vec![DbNotificationKind::CommentReply, DbNotificationKind::PostReply],
                "activity" => vec![DbNotificationKind::ModAction, DbNotificationKind::System],
                other => {
                    other
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
                        .collect()
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

        // Collect all referenced IDs for batch loading
        let actor_ids: Vec<Uuid> = db_notifications.iter()
            .filter_map(|n| n.actor_user_id)
            .collect();
        let comment_ids: Vec<Uuid> = db_notifications.iter()
            .filter_map(|n| n.comment_id)
            .collect();
        let post_ids: Vec<Uuid> = db_notifications.iter()
            .filter_map(|n| n.post_id)
            .collect();
        let message_ids: Vec<Uuid> = db_notifications.iter()
            .filter_map(|n| n.message_id)
            .collect();

        // Batch load actors
        let actors: Vec<(Uuid, String, Option<String>, Option<String>)> = if !actor_ids.is_empty() {
            users::table
                .filter(users::id.eq_any(&actor_ids))
                .select((users::id, users::name, users::display_name, users::avatar))
                .load::<(Uuid, String, Option<String>, Option<String>)>(conn)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Batch load comments with their post titles and board names
        let comment_data: Vec<(Uuid, String, Uuid, String, String)> = if !comment_ids.is_empty() {
            comments::table
                .inner_join(posts::table.on(posts::id.eq(comments::post_id)))
                .inner_join(boards::table.on(boards::id.eq(comments::board_id)))
                .filter(comments::id.eq_any(&comment_ids))
                .select((
                    comments::id,
                    comments::body,
                    comments::post_id,
                    posts::title,
                    boards::name,
                ))
                .load::<(Uuid, String, Uuid, String, String)>(conn)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Batch load posts with board names
        let post_data: Vec<(Uuid, String, Uuid, String)> = if !post_ids.is_empty() {
            posts::table
                .inner_join(boards::table.on(boards::id.eq(posts::board_id)))
                .filter(posts::id.eq_any(&post_ids))
                .select((posts::id, posts::title, posts::board_id, boards::name))
                .load::<(Uuid, String, Uuid, String)>(conn)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Batch load messages
        let message_data: Vec<(Uuid, String)> = if !message_ids.is_empty() {
            private_messages::table
                .filter(private_messages::id.eq_any(&message_ids))
                .select((private_messages::id, private_messages::body))
                .load::<(Uuid, String)>(conn)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Build enriched notifications
        let enriched: Vec<Notification> = db_notifications.into_iter().map(|n| {
            let actor = n.actor_user_id.and_then(|aid| {
                actors.iter().find(|a| a.0 == aid).map(|a| NotificationActor {
                    id: a.0.to_string().into(),
                    name: a.1.clone(),
                    display_name: a.2.clone(),
                    avatar: a.3.clone(),
                })
            });

            let comment = n.comment_id.and_then(|cid| {
                comment_data.iter().find(|c| c.0 == cid).map(|c| NotificationCommentContext {
                    id: c.0.to_string().into(),
                    body: truncate_snippet(&c.1, 120),
                    post_id: c.2.to_string().into(),
                    post_title: c.3.clone(),
                    board_name: c.4.clone(),
                })
            });

            let post = n.post_id.and_then(|pid| {
                post_data.iter().find(|p| p.0 == pid).map(|p| NotificationPostContext {
                    id: p.0.to_string().into(),
                    title: p.1.clone(),
                    board_name: p.3.clone(),
                    board_id: p.2.to_string().into(),
                })
            });

            let message = n.message_id.and_then(|mid| {
                message_data.iter().find(|m| m.0 == mid).map(|m| NotificationMessageContext {
                    id: m.0.to_string().into(),
                    body: truncate_snippet(&m.1, 120),
                })
            });

            Notification {
                id: n.id.to_string().into(),
                kind: kind_to_str(&n.kind).to_string(),
                is_read: n.is_read,
                created_at: n.created_at.to_string(),
                comment_id: n.comment_id.map(|id| id.to_string().into()),
                post_id: n.post_id.map(|id| id.to_string().into()),
                message_id: n.message_id.map(|id| id.to_string().into()),
                actor,
                post,
                comment,
                message,
            }
        }).collect();

        Ok(enriched)
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

        let total: i64 = notifications::table
            .filter(notifications::recipient_user_id.eq(user.id))
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

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

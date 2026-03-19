use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbNotificationKind,
    models::{
        message::message::PrivateMessageInsertForm,
        notification::notifications::NotificationInsertForm,
    },
    schema::{private_messages, notifications, user_blocks, site},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::message::PrivateMessage,
    LoggedInUser,
    utils::emoji::process_content_with_emojis,
    Settings,
};

#[derive(Default)]
pub struct SendMessageMutations;

#[derive(InputObject)]
pub struct SendMessageInput {
    pub recipient_id: ID,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(SimpleObject)]
pub struct SendMessageResponse {
    pub message: PrivateMessage,
}

#[Object]
impl SendMessageMutations {
    /// Send a private message to another user
    pub async fn send_message(
        &self,
        ctx: &Context<'_>,
        input: SendMessageInput,
    ) -> Result<SendMessageResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user_approved(pool).await?;
        let conn = &mut get_conn(pool).await?;

        let recipient_id: Uuid = input.recipient_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid recipient ID".into()))?;

        // Check if sender is blocked by recipient
        let is_blocked: bool = user_blocks::table
            .filter(user_blocks::user_id.eq(recipient_id))
            .filter(user_blocks::target_id.eq(user.id))
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
            > 0;

        if is_blocked {
            return Err(TinyBoardsError::from_message(403, "You are blocked by this user").into());
        }

        // Check if sender has blocked recipient
        let has_blocked: bool = user_blocks::table
            .filter(user_blocks::user_id.eq(user.id))
            .filter(user_blocks::target_id.eq(recipient_id))
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
            > 0;

        if has_blocked {
            return Err(TinyBoardsError::from_message(403, "You have blocked this user").into());
        }

        // Validate input
        if input.body.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Message body cannot be empty").into());
        }

        if input.body.len() > 10000 {
            return Err(TinyBoardsError::from_message(400, "Message too long").into());
        }

        if let Some(ref subject) = input.subject {
            if subject.len() > 200 {
                return Err(TinyBoardsError::from_message(400, "Subject too long").into());
            }
        }

        // Process content with emojis
        let settings = ctx.data::<Settings>()?.as_ref();

        // Read site config for emoji settings
        let site_config: tinyboards_db::models::site::site::Site = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let emoji_limit = if site_config.emoji_enabled {
            site_config.max_emojis_per_comment.map(|limit| limit as usize)
        } else {
            Some(0)
        };

        let body_html = process_content_with_emojis(
            &input.body,
            pool,
            None, // No board context for private messages
            settings,
            emoji_limit,
        )
        .await?;

        // Create message
        let form = PrivateMessageInsertForm {
            creator_id: user.id,
            recipient_id: Some(recipient_id),
            recipient_board_id: None,
            subject: input.subject,
            body: input.body,
            body_html,
            is_read: false,
            is_sender_hidden: false,
        };

        let message: tinyboards_db::models::message::message::PrivateMessage =
            diesel::insert_into(private_messages::table)
                .values(&form)
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Create notification for recipient
        let notif_form = NotificationInsertForm {
            kind: DbNotificationKind::PrivateMessage,
            recipient_user_id: recipient_id,
            comment_id: None,
            post_id: None,
            message_id: Some(message.id),
            is_read: false,
        };

        diesel::insert_into(notifications::table)
            .values(&notif_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(SendMessageResponse {
            message: PrivateMessage::from(message),
        })
    }
}

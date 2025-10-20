use async_graphql::*;
use tinyboards_db::{
    models::{
        message::message::{Message as DbMessage, MessageForm, MessageNotif, MessageNotifForm},
        user::user_blocks::UserBlock,
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::{TinyBoardsError, parser::parse_markdown_opt};

use crate::{
    structs::message::Message,
    LoggedInUser,
};

#[derive(Default)]
pub struct SendMessageMutations;

#[derive(InputObject)]
pub struct SendMessageInput {
    pub recipient_id: i32,
    pub title: String,
    pub body: String,
}

#[derive(SimpleObject)]
pub struct SendMessageResponse {
    pub message: Message,
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

        // Check if sender is blocked by recipient
        let is_blocked = UserBlock::is_blocked(pool, input.recipient_id, user.id).await.unwrap_or(false);
        if is_blocked {
            return Err(TinyBoardsError::from_message(403, "You are blocked by this user").into());
        }

        // Check if recipient has blocked sender
        let has_blocked = UserBlock::is_blocked(pool, user.id, input.recipient_id).await.unwrap_or(false);
        if has_blocked {
            return Err(TinyBoardsError::from_message(403, "You have blocked this user").into());
        }

        // Validate input
        if input.title.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Title cannot be empty").into());
        }

        if input.body.trim().is_empty() {
            return Err(TinyBoardsError::from_message(400, "Message body cannot be empty").into());
        }

        if input.title.len() > 200 {
            return Err(TinyBoardsError::from_message(400, "Title too long").into());
        }

        if input.body.len() > 10000 {
            return Err(TinyBoardsError::from_message(400, "Message too long").into());
        }

        // Parse markdown
        let body_html = parse_markdown_opt(&input.body);

        // Create message
        let message_form = MessageForm {
            creator_id: Some(user.id),
            recipient_user_id: Some(Some(input.recipient_id)),
            recipient_board_id: Some(None),
            title: Some(input.title),
            body: Some(input.body),
            body_html,
            ..Default::default()
        };

        let message = DbMessage::create(pool, &message_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to send message"))?;

        // Create notification for recipient
        let notif_form = MessageNotifForm {
            recipient_id: Some(input.recipient_id),
            pm_id: Some(message.id),
            read: Some(false),
        };

        MessageNotif::create(pool, &notif_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create notification"))?;

        // TODO: Send notification
        // Notification::send(pool, NotificationType::Message(message.id), input.recipient_id).await
        //     .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to send notification"))?;

        Ok(SendMessageResponse {
            message: Message::from(message),
        })
    }
}
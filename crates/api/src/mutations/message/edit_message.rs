use async_graphql::*;
use tinyboards_db::{
    models::message::message::{Message as DbMessage, MessageForm},
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::{TinyBoardsError, parser::parse_markdown_opt};

use crate::{
    structs::message::Message,
    LoggedInUser,
};

#[derive(Default)]
pub struct EditMessageMutations;

#[derive(InputObject)]
pub struct EditMessageInput {
    pub message_id: i32,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(SimpleObject)]
pub struct EditMessageResponse {
    pub message: Message,
}

#[Object]
impl EditMessageMutations {
    /// Edit a private message (only sender can edit)
    pub async fn edit_message(
        &self,
        ctx: &Context<'_>,
        input: EditMessageInput,
    ) -> Result<EditMessageResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Check if message exists and user owns it
        let existing_message = DbMessage::read(pool, input.message_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Message not found"))?;

        if existing_message.creator_id != user.id {
            return Err(TinyBoardsError::from_message(403, "You can only edit your own messages").into());
        }

        // Validate input if provided
        if let Some(ref title) = input.title {
            if title.trim().is_empty() {
                return Err(TinyBoardsError::from_message(400, "Title cannot be empty").into());
            }
            if title.len() > 200 {
                return Err(TinyBoardsError::from_message(400, "Title too long").into());
            }
        }

        if let Some(ref body) = input.body {
            if body.trim().is_empty() {
                return Err(TinyBoardsError::from_message(400, "Message body cannot be empty").into());
            }
            if body.len() > 10000 {
                return Err(TinyBoardsError::from_message(400, "Message too long").into());
            }
        }

        // Parse markdown for body if provided
        let body_html = if let Some(ref body) = input.body {
            parse_markdown_opt(body)
        } else {
            None
        };

        // Create update form
        let message_form = MessageForm {
            title: input.title,
            body: input.body,
            body_html,
            updated: Some(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let updated_message = DbMessage::update(pool, input.message_id, &message_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update message"))?;

        Ok(EditMessageResponse {
            message: Message::from(updated_message),
        })
    }
}
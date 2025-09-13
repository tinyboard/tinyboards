use async_graphql::*;
use tinyboards_db::{
    models::message::message::Message as DbMessage,
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct MessageActionMutations;

#[Object]
impl MessageActionMutations {
    /// Mark a conversation as read
    pub async fn mark_conversation_read(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let current_user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        DbMessage::mark_conversation_read(pool, current_user.person.id, user_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to mark conversation as read"))?;

        Ok(true)
    }

    /// Delete a message (only sender can delete)
    pub async fn delete_message(
        &self,
        ctx: &Context<'_>,
        message_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Check if user owns the message
        let message = DbMessage::read(pool, message_id).await
            .map_err(|_| TinyBoardsError::from_message(404, "Message not found"))?;

        if message.creator_id != user.person.id {
            return Err(TinyBoardsError::from_message(403, "You can only delete your own messages").into());
        }

        DbMessage::delete(pool, message_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete message"))?;

        Ok(true)
    }
}
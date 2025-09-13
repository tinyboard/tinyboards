use async_graphql::*;
use tinyboards_db::{
    models::{
        message::message::Message as DbMessage,
        person::person::Person as DbPerson,
    },
    utils::DbPool,
};

use crate::{
    structs::{message::{Conversation, Message}, person::Person},
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryMessages;

#[Object]
impl QueryMessages {
    /// Get all conversations for the current user
    pub async fn list_conversations(&self, ctx: &Context<'_>) -> Result<Vec<Conversation>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let conversations_data = DbMessage::list_conversations_for_user(pool, user.person.id).await
            .map_err(|e| Error::new(format!("Failed to load conversations: {}", e)))?;

        let mut conversations = Vec::new();
        
        for (other_user_id, last_message) in conversations_data {
            // Get the other user's info
            let other_user = DbPerson::get_user_by_id(pool, other_user_id).await
                .map_err(|e| Error::new(format!("Failed to load user: {}", e)))?;

            // Get unread count for this conversation
            let unread_count = DbMessage::get_unread_count_for_user(pool, user.person.id).await
                .map_err(|e| Error::new(format!("Failed to get unread count: {}", e)))? as i32;

            conversations.push(Conversation {
                other_user: Person::from(other_user),
                last_message: Message::from(last_message.clone()),
                unread_count,
                last_activity: last_message.published.to_string(),
            });
        }

        Ok(conversations)
    }

    /// Get messages in a conversation with another user
    pub async fn get_conversation(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>> {
        let pool = ctx.data::<DbPool>()?;
        let current_user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        // Mark conversation as read
        DbMessage::mark_conversation_read(pool, current_user.person.id, user_id).await
            .map_err(|e| Error::new(format!("Failed to mark conversation as read: {}", e)))?;

        let messages = DbMessage::list_messages_between_users(
            pool,
            current_user.person.id,
            user_id,
            limit,
            offset,
        ).await
        .map_err(|e| Error::new(format!("Failed to load messages: {}", e)))?;

        Ok(messages.into_iter().map(Message::from).collect())
    }

    /// Get unread message count for current user
    pub async fn get_unread_message_count(&self, ctx: &Context<'_>) -> Result<i32> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let count = DbMessage::get_unread_count_for_user(pool, user.person.id).await
            .map_err(|e| Error::new(format!("Failed to get unread count: {}", e)))?;

        Ok(count as i32)
    }
}
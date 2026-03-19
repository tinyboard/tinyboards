use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::message::message::PrivateMessage as DbPrivateMessage,
    schema::private_messages,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct MessageActionMutations;

#[Object]
impl MessageActionMutations {
    /// Mark specific messages as read
    pub async fn mark_messages_read(
        &self,
        ctx: &Context<'_>,
        message_ids: Vec<ID>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let current_user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;

        let uuids: Vec<Uuid> = message_ids
            .iter()
            .map(|id| id.parse::<Uuid>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid message ID"))?;

        // Only mark messages where the current user is the recipient
        diesel::update(
            private_messages::table
                .filter(private_messages::id.eq_any(&uuids))
                .filter(private_messages::recipient_id.eq(current_user.id))
        )
        .set(private_messages::is_read.eq(true))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Delete a message (soft delete via deleted_at)
    /// Sender or recipient can delete; both parties see it disappear
    pub async fn delete_message(
        &self,
        ctx: &Context<'_>,
        message_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;

        let msg_id: Uuid = message_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid message ID".into()))?;

        // Check if user is sender or recipient
        let message: DbPrivateMessage = private_messages::table
            .find(msg_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Message not found".into()))?;

        let is_sender = message.creator_id == user.id;
        let is_recipient = message.recipient_id == Some(user.id);

        if !is_sender && !is_recipient {
            return Err(TinyBoardsError::from_message(403, "You can only delete your own messages").into());
        }

        // Soft delete the message
        diesel::update(private_messages::table.find(msg_id))
            .set(private_messages::deleted_at.eq(chrono::Utc::now()))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}

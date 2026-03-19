use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::message::message::PrivateMessage as DbPrivateMessage,
    schema::{private_messages, users},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::message::{Conversation, PrivateMessage},
    structs::user::User,
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryMessages;

#[Object]
impl QueryMessages {
    /// Get all conversations for the current user (grouped by conversation partner)
    pub async fn list_conversations(&self, ctx: &Context<'_>) -> Result<Vec<Conversation>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;
        let my_id = user.id;

        // Get distinct conversation partners
        // A conversation partner is anyone we've sent messages to or received messages from
        // We get the latest message for each partner
        let sent_partners: Vec<Uuid> = private_messages::table
            .filter(private_messages::creator_id.eq(my_id))
            .filter(private_messages::deleted_at.is_null())
            .filter(private_messages::recipient_id.is_not_null())
            .select(private_messages::recipient_id)
            .distinct()
            .load::<Option<Uuid>>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
            .into_iter()
            .flatten()
            .collect();

        let received_partners: Vec<Uuid> = private_messages::table
            .filter(private_messages::recipient_id.eq(my_id))
            .filter(private_messages::deleted_at.is_null())
            .select(private_messages::creator_id)
            .distinct()
            .load::<Uuid>(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Combine and deduplicate partners
        let mut partners: Vec<Uuid> = sent_partners;
        for p in received_partners {
            if !partners.contains(&p) {
                partners.push(p);
            }
        }

        let mut conversations = Vec::new();

        for partner_id in partners {
            // Get latest message in this conversation
            let last_msg: DbPrivateMessage = private_messages::table
                .filter(
                    private_messages::creator_id.eq(my_id)
                        .and(private_messages::recipient_id.eq(partner_id))
                        .or(
                            private_messages::creator_id.eq(partner_id)
                                .and(private_messages::recipient_id.eq(my_id))
                        )
                )
                .filter(private_messages::deleted_at.is_null())
                .order(private_messages::created_at.desc())
                .first(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            // Count unread messages from this partner
            let unread: i64 = private_messages::table
                .filter(private_messages::creator_id.eq(partner_id))
                .filter(private_messages::recipient_id.eq(my_id))
                .filter(private_messages::is_read.eq(false))
                .filter(private_messages::deleted_at.is_null())
                .count()
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            // Get partner user info
            let partner_user: tinyboards_db::models::user::user::User = users::table
                .find(partner_id)
                .first(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            let last_activity = last_msg.created_at.to_string();

            conversations.push(Conversation {
                other_user: User::from_db(partner_user, None),
                last_message: PrivateMessage::from(last_msg),
                unread_count: unread as i32,
                last_activity,
            });
        }

        // Sort by most recent activity
        conversations.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(conversations)
    }

    /// Get messages in a conversation with another user
    pub async fn get_conversation(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<PrivateMessage>> {
        let pool = ctx.data::<DbPool>()?;
        let current_user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;
        let my_id = current_user.id;

        let other_id: Uuid = user_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid user ID".into()))?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        // Mark messages from the other user as read
        diesel::update(
            private_messages::table
                .filter(private_messages::creator_id.eq(other_id))
                .filter(private_messages::recipient_id.eq(my_id))
                .filter(private_messages::is_read.eq(false))
        )
        .set(private_messages::is_read.eq(true))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Get messages between the two users
        let messages: Vec<DbPrivateMessage> = private_messages::table
            .filter(
                private_messages::creator_id.eq(my_id)
                    .and(private_messages::recipient_id.eq(other_id))
                    .or(
                        private_messages::creator_id.eq(other_id)
                            .and(private_messages::recipient_id.eq(my_id))
                    )
            )
            .filter(private_messages::deleted_at.is_null())
            .order(private_messages::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(messages.into_iter().map(PrivateMessage::from).collect())
    }

    /// Get unread message count for current user
    pub async fn get_unread_message_count(&self, ctx: &Context<'_>) -> Result<i32> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let conn = &mut get_conn(pool).await?;

        let count: i64 = private_messages::table
            .filter(private_messages::recipient_id.eq(user.id))
            .filter(private_messages::is_read.eq(false))
            .filter(private_messages::deleted_at.is_null())
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(count as i32)
    }
}

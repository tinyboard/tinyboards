use crate::models::message::message::{MessageNotif, MessageNotifForm};
use crate::schema::{pm_notif, private_message};
use crate::{
    models::message::message::{Message, MessageForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl, ExpressionMethods, BoolExpressionMethods};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl Message {
    pub async fn submit(pool: &DbPool, form: MessageForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit message"))
    }

    pub async fn list_conversations_for_user(pool: &DbPool, user_id: i32) -> Result<Vec<(i32, Self)>, Error> {
        let conn = &mut get_conn(pool).await?;
        
        // Get all messages for this user (sent or received)
        let messages = private_message::table
            .filter(
                private_message::creator_id.eq(user_id)
                .or(private_message::recipient_user_id.eq(user_id))
            )
            .filter(private_message::recipient_user_id.is_not_null()) // Only user-to-user messages
            .order_by(private_message::published.desc())
            .load::<Self>(conn)
            .await?;
        
        // Group by conversation partner and get the latest message for each
        let mut conversations: std::collections::HashMap<i32, Self> = std::collections::HashMap::new();
        
        for message in messages {
            let other_user_id = if message.creator_id == user_id {
                message.recipient_user_id.unwrap_or(0)
            } else {
                message.creator_id
            };
            
            if other_user_id != 0 && !conversations.contains_key(&other_user_id) {
                conversations.insert(other_user_id, message);
            }
        }
        
        let mut result: Vec<(i32, Self)> = conversations.into_iter().collect();
        result.sort_by(|a, b| b.1.published.cmp(&a.1.published));
        
        Ok(result)
    }

    pub async fn list_messages_between_users(
        pool: &DbPool, 
        user1_id: i32, 
        user2_id: i32,
        limit: Option<i64>,
        offset: Option<i64>
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        
        let mut query = private_message::table
            .filter(
                (private_message::creator_id.eq(user1_id).and(private_message::recipient_user_id.eq(user2_id)))
                .or(private_message::creator_id.eq(user2_id).and(private_message::recipient_user_id.eq(user1_id)))
            )
            .filter(private_message::recipient_user_id.is_not_null()) // Only user-to-user messages
            .order_by(private_message::published.desc())
            .into_boxed();
            
        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        
        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }
        
        query.load::<Self>(conn).await
    }

    pub async fn get_unread_count_for_user(pool: &DbPool, user_id: i32) -> Result<i64, Error> {
        use diesel::dsl::count;
        
        let conn = &mut get_conn(pool).await?;
        
        pm_notif::table
            .filter(pm_notif::recipient_id.eq(user_id))
            .filter(pm_notif::read.eq(false))
            .select(count(pm_notif::id))
            .first::<i64>(conn)
            .await
    }

    pub async fn mark_conversation_read(pool: &DbPool, user_id: i32, other_user_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        
        // Get message IDs from this conversation where user is recipient
        let message_ids: Vec<i32> = private_message::table
            .select(private_message::id)
            .filter(private_message::creator_id.eq(other_user_id))
            .filter(private_message::recipient_user_id.eq(user_id))
            .load::<i32>(conn)
            .await?;
        
        if message_ids.is_empty() {
            return Ok(0);
        }
        
        // Update pm_notif records for these messages
        diesel::update(pm_notif::table)
            .filter(pm_notif::recipient_id.eq(user_id))
            .filter(pm_notif::read.eq(false))
            .filter(pm_notif::pm_id.eq_any(message_ids))
            .set(pm_notif::read.eq(true))
            .execute(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for Message {
    type Form = MessageForm;
    type IdType = i32;

    async fn read(pool: &DbPool, message_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        private_message::table.find(message_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, message_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(private_message::table.find(message_id))
            .execute(conn)
            .await
    }
    async fn create(pool: &DbPool, form: &MessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_message = diesel::insert_into(private_message::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new_message)
    }
    async fn update(pool: &DbPool, message_id: i32, form: &MessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(private_message::table.find(message_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for MessageNotif {
    type Form = MessageNotifForm;
    type IdType = i32;

    async fn read(pool: &DbPool, notif_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        pm_notif::table.find(notif_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, notif_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(pm_notif::table.find(notif_id)).execute(conn).await
    }
    async fn create(pool: &DbPool, form: &MessageNotifForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_notif = diesel::insert_into(pm_notif::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new_notif)
    }
    async fn update(pool: &DbPool, notif_id: i32, form: &MessageNotifForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(pm_notif::table.find(notif_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

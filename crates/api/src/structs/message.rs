use async_graphql::*;
use tinyboards_db::models::message::message::Message as DbMessage;

use crate::structs::user::User;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Message {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: String,
    pub updated: Option<String>,
    #[graphql(skip)]
    pub creator_id: i32,
    #[graphql(skip)]
    pub recipient_user_id: Option<i32>,
}

#[ComplexObject]
impl Message {
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<User> {
        use tinyboards_db::{models::user::user::User as DbUser, utils::DbPool};
        let pool = ctx.data::<DbPool>()?;

        let creator = DbUser::get_by_id(pool, self.creator_id).await
            .map_err(|e| Error::new(format!("Failed to load message creator: {}", e)))?;

        Ok(User::from(creator))
    }

    pub async fn recipient(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        use tinyboards_db::{models::user::user::User as DbUser, utils::DbPool};
        let pool = ctx.data::<DbPool>()?;

        if let Some(recipient_id) = self.recipient_user_id {
            let recipient = DbUser::get_by_id(pool, recipient_id).await
                .map_err(|e| Error::new(format!("Failed to load message recipient: {}", e)))?;
            Ok(Some(User::from(recipient)))
        } else {
            Ok(None)
        }
    }
}

#[derive(SimpleObject)]
pub struct Conversation {
    pub other_user: User,
    pub last_message: Message,
    pub unread_count: i32,
    pub last_activity: String,
}

impl From<DbMessage> for Message {
    fn from(message: DbMessage) -> Self {
        Self {
            id: message.id,
            title: message.title,
            body: message.body,
            published: message.published.to_string(),
            updated: message.updated.map(|u| u.to_string()),
            creator_id: message.creator_id,
            recipient_user_id: message.recipient_user_id,
        }
    }
}
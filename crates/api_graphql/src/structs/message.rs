use async_graphql::*;
use tinyboards_db::models::message::message::Message as DbMessage;

use crate::structs::person::Person;

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
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Person> {
        use tinyboards_db::{models::person::person::Person as DbPerson, utils::DbPool};
        let pool = ctx.data::<DbPool>()?;
        
        let creator = DbPerson::get_user_by_id(pool, self.creator_id).await
            .map_err(|e| Error::new(format!("Failed to load message creator: {}", e)))?;
            
        Ok(Person::from(creator))
    }

    pub async fn recipient(&self, ctx: &Context<'_>) -> Result<Option<Person>> {
        use tinyboards_db::{models::person::person::Person as DbPerson, utils::DbPool};
        let pool = ctx.data::<DbPool>()?;
        
        if let Some(recipient_id) = self.recipient_user_id {
            let recipient = DbPerson::get_user_by_id(pool, recipient_id).await
                .map_err(|e| Error::new(format!("Failed to load message recipient: {}", e)))?;
            Ok(Some(Person::from(recipient)))
        } else {
            Ok(None)
        }
    }
}

#[derive(SimpleObject)]
pub struct Conversation {
    pub other_user: Person,
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
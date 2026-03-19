use async_graphql::*;
use async_graphql::dataloader::DataLoader;
use tinyboards_db::models::message::message::PrivateMessage as DbPrivateMessage;

use crate::{
    newtypes::UserId,
    PostgresLoader,
    structs::user::User,
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PrivateMessage {
    pub id: ID,
    pub subject: Option<String>,
    pub body: String,
    pub body_html: String,
    pub is_read: bool,
    pub is_sender_hidden: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
    pub is_deleted: bool,

    #[graphql(skip)]
    pub uuid_id: uuid::Uuid,
    #[graphql(skip)]
    pub uuid_creator_id: uuid::Uuid,
    #[graphql(skip)]
    pub uuid_recipient_id: Option<uuid::Uuid>,
}

#[ComplexObject]
impl PrivateMessage {
    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        Ok(loader.load_one(UserId(self.uuid_creator_id)).await?)
    }

    pub async fn recipient(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        if let Some(rid) = self.uuid_recipient_id {
            let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
            Ok(loader.load_one(UserId(rid)).await?)
        } else {
            Ok(None)
        }
    }
}

#[derive(SimpleObject)]
pub struct Conversation {
    pub other_user: User,
    pub last_message: PrivateMessage,
    pub unread_count: i32,
    #[graphql(name = "lastActivity")]
    pub last_activity: String,
}

impl From<DbPrivateMessage> for PrivateMessage {
    fn from(msg: DbPrivateMessage) -> Self {
        Self {
            id: msg.id.to_string().into(),
            subject: msg.subject.clone(),
            body: msg.body.clone(),
            body_html: msg.body_html.clone(),
            is_read: msg.is_read,
            is_sender_hidden: msg.is_sender_hidden,
            created_at: msg.created_at.to_string(),
            updated_at: msg.updated_at.to_string(),
            is_deleted: msg.deleted_at.is_some(),
            uuid_id: msg.id,
            uuid_creator_id: msg.creator_id,
            uuid_recipient_id: msg.recipient_id,
        }
    }
}

use crate::schema::private_message::dsl::*;
use crate::{
    models::message::message::{Message, MessageForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl Message {
    pub async fn submit(pool: &DbPool, form: MessageForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit message"))
    }
}

#[async_trait::async_trait]
impl Crud for Message {
    type Form = MessageForm;
    type IdType = i32;

    async fn read(pool: &DbPool, message_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        private_message.find(message_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, message_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(private_message.find(message_id))
            .execute(conn)
            .await
    }
    async fn create(pool: &DbPool, form: &MessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_message = diesel::insert_into(private_message)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new_message)
    }
    async fn update(pool: &DbPool, message_id: i32, form: &MessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(private_message.find(message_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

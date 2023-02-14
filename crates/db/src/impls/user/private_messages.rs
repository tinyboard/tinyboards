use crate::{
    schema::private_messages::dsl::*,
    models::user::private_messages::{PrivateMessage, PrivateMessageForm},
    traits::Crud, utils::{DbPool, get_conn},
};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for PrivateMessage {
    type Form = PrivateMessageForm;
    type IdType = i32;
    async fn read(pool: &DbPool, pm_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        private_messages.find(pm_id).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, pm_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(private_messages.find(pm_id)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &PrivateMessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let pm = diesel::insert_into(private_messages)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(pm)
    }
    async fn update(pool: &DbPool, pm_id: i32, form: &PrivateMessageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(private_messages.find(pm_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

}
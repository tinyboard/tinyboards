use crate::{
    models::local_user::person_mentions::{UserMention, UserMentionForm},
    traits::Crud, utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for UserMention {
    type Form = UserMentionForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_mentions::dsl::*;
        user_mentions.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_mentions::dsl::*;
        diesel::delete(user_mentions.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_mentions::dsl::*;
        let new = diesel::insert_into(user_mentions)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_mentions::dsl::*;
        diesel::update(user_mentions.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}
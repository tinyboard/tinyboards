use crate::{
    schema::emoji::dsl::emoji,
    models::emoji::emoji::{Emoji, EmojiForm},
    utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Emoji {
    pub async fn create(pool: &DbPool, form: &EmojiForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(emoji)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update(pool: &DbPool, emoji_id: i32, form: &EmojiForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(emoji.find(emoji_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn delete(pool: &DbPool, emoji_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(emoji.find(emoji_id))
            .execute(conn)
            .await
    }
}
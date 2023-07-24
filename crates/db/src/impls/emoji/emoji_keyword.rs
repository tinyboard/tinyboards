use crate::{
    schema::emoji_keyword::dsl::{emoji_keyword, emoji_id},
    models::emoji::emoji_keyword::{EmojiKeyword, EmojiKeywordForm},
    utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl EmojiKeyword {
    pub async fn create(pool: &mut DbPool, forms: Vec<EmojiKeywordForm>) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(emoji_keyword)
            .values(forms)
            .get_results::<Self>(conn)
            .await
    }

    pub async fn delete(pool: &mut DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(emoji_keyword.filter(emoji_id.eq(id_)))
            .execute(conn)
            .await
    }
}
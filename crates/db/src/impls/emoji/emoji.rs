use crate::{
    schema::emoji::dsl::*,
    models::emoji::emoji::{Emoji, EmojiForm},
    utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl, ExpressionMethods, BoolExpressionMethods};
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

    pub async fn list_site_emojis(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        emoji
            .filter(emoji_scope.eq("site"))
            .filter(is_active.eq(true))
            .order(shortcode.asc())
            .load::<Self>(conn)
            .await
    }

    pub async fn list_board_emojis(pool: &DbPool, board_id_param: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        emoji
            .filter(board_id.eq(board_id_param))
            .filter(is_active.eq(true))
            .order(shortcode.asc())
            .load::<Self>(conn)
            .await
    }

    pub async fn list_all_available_emojis(pool: &DbPool, board_id_param: Option<i32>) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = emoji
            .filter(is_active.eq(true))
            .into_boxed();

        if let Some(board_id_param) = board_id_param {
            query = query.filter(
                emoji_scope.eq("site").or(board_id.eq(board_id_param))
            );
        } else {
            query = query.filter(emoji_scope.eq("site"));
        }

        query.order(shortcode.asc()).load::<Self>(conn).await
    }

    pub async fn increment_usage(pool: &DbPool, emoji_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(emoji.find(emoji_id))
            .set(usage_count.eq(usage_count + 1))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read(pool: &DbPool, emoji_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        emoji.find(emoji_id).first::<Self>(conn).await
    }

    pub async fn list_all_site_admin(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        emoji
            .filter(emoji_scope.eq("site"))
            .order(shortcode.asc())
            .load::<Self>(conn)
            .await
    }

    pub async fn list_all_for_board_admin(pool: &DbPool, board_id_param: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        emoji
            .filter(board_id.eq(board_id_param))
            .order(shortcode.asc())
            .load::<Self>(conn)
            .await
    }
}
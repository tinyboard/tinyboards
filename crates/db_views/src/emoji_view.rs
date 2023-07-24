use crate::structs::EmojiView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    schema::{emoji, emoji_keyword},
    models::emoji::{emoji::Emoji, emoji_keyword::EmojiKeyword},
    utils::{get_conn, DbPool},
};
use std::collections::HashMap;

type EmojiViewTuple = (
    Emoji,
    Option<EmojiKeyword>,
);

impl EmojiView {
    pub async fn get(pool: &mut DbPool, emoji_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let emojis = emoji::table
            .find(emoji_id)
            .left_join(
                emoji_keyword::table.on(
                    emoji_keyword::emoji_id.eq(emoji::id)
                ),
            )
            .select((
                emoji::all_columns,
                emoji_keyword::all_columns.nullable(),
            ))
            .load::<EmojiViewTuple>(conn)
            .await?;

        if let Some(emoji) = EmojiView::from_tuple_to_vec(emojis)
            .into_iter()
            .next() {
                Ok(emoji)
            } else {
                Err(diesel::result::Error::NotFound)
            }
    }

    pub async fn get_all(pool: &mut DbPool, for_local_site_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let emojis = emoji::table
            .filter(emoji::local_site_id.eq(for_local_site_id))
            .left_join(
                emoji_keyword::table.on(
                    emoji_keyword::emoji_id.eq(emoji::id)
                ),
            )
            .select((
                emoji::all_columns,
                emoji_keyword::all_columns.nullable(),
            ))
            .load::<EmojiViewTuple>(conn)
            .await?;

        Ok(EmojiView::from_tuple_to_vec(emojis))
    }

    fn from_tuple_to_vec(items: Vec<EmojiViewTuple>) -> Vec<Self> {
        let mut result = Vec::new();
        let mut hash: HashMap<i32, Vec<EmojiKeyword>> = HashMap::new();
        for item in &items {
            let emoji_id: i32 = item.0.id;
            
            if let std::collections::hash_map::Entry::Vacant(e) = hash.entry(emoji_id) {
                e.insert(Vec::new());
                result.push(EmojiView {
                    emoji: item.0.clone(),
                    keywords: Vec::new(),
                });
            }

            if let Some(item_keyword) = &item.1 {
                if let Some(keywords) = hash.get_mut(&emoji_id) {
                    keywords.push(item_keyword.clone());
                }
            }

            for emoji in &mut result {
                if let Some(keywords) = hash.get_mut(&emoji.emoji.id) {
                    emoji.keywords = keywords.clone();
                }
            }
        }
        result
    }
}


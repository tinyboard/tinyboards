use crate::schema::emoji_keyword;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = emoji_keyword)]
pub struct EmojiKeyword {
    pub id: i32,
    pub emoji_id: i32,
    pub keyword: String,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = emoji_keyword)]
pub struct EmojiKeywordForm {
    pub emoji_id: Option<i32>,
    pub keyword: Option<String>,
}
use crate::{schema::emoji, newtypes::DbUrl};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = emoji)]
pub struct Emoji {
    pub id: i32,
    pub local_site_id: i32,
    pub shortcode: String,
    pub image_url: DbUrl,
    pub alt_text: String,
    pub category: String,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = emoji)]
pub struct EmojiForm {
    pub local_site_id: Option<i32>,
    pub shortcode: Option<String>,
    pub image_url: Option<DbUrl>,
    pub alt_text: Option<String>,
    pub category: Option<String>,
    pub updated: Option<NaiveDateTime>,
}
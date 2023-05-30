use crate::schema::person_mentions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person_mentions)]
pub struct PersonMention {
    pub id: i32,
    pub recipient_id: i32,
    pub comment_id: i32,
    pub read: bool,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = person_mentions)]
pub struct PersonMentionForm {
    pub recipient_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub read: Option<bool>,
}
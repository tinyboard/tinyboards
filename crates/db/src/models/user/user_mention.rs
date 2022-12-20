use crate::schema::user_mentions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_mentions)]
pub struct UserMention {
    pub id: i32,
    pub recipient_id: i32,
    pub comment_id: i32,
    pub read: bool,
    pub creation_date: NaiveDateTime,
}

use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_mention;
use chrono::NaiveDateTime;


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_mention)]
pub struct UserMention {
    pub id: i32,
    pub recipient_id: i32,
    pub comment_id: i32,
    pub read: bool,
    pub published: NaiveDateTime,
}
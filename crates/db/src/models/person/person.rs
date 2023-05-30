use crate::schema::person;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub login_nonce: Option<i32>,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<String>,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub signature: Option<String>,
    pub actor_id: String,
    pub local: bool,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub inbox_url: String,
    pub shared_inbox_url: Option<String>,
    pub bot_account: bool,
    pub last_refreshed_date: NaiveDateTime,
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Default)]
#[diesel(table_name = person)]
pub struct PersonSafe {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub is_banned: bool,
    pub local: bool,
    pub actor_id: Option<String>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<String>,
    pub signature: Option<String>,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub inbox_url: String,
    pub shared_inbox_url: Option<String>,
    pub bot_account: bool,
}


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = person)]
pub struct PersonForm {
    pub name: Option<String>,
    pub display_name: Option<Option<String>>,
    pub passhash: Option<String>,
    pub email: Option<String>,
    pub login_nonce: Option<Option<i32>>,
    pub is_banned: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<Option<String>>,
    pub signature: Option<Option<String>>,
    pub is_deleted: Option<bool>,
    pub unban_date: Option<Option<NaiveDateTime>>,
    pub banner: Option<Option<String>>,
    pub bio: Option<Option<String>>,
    pub actor_id: Option<String>,
    pub local: Option<bool>,
    pub private_key: Option<Option<String>>,
    pub public_key: Option<Option<String>>,
    pub inbox_url: Option<String>,
    pub shared_inbox_url: Option<Option<String>>,
    pub bot_account: Option<bool>,
    pub last_refreshed_date: Option<NaiveDateTime>,
}
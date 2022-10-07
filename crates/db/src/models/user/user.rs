use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_;

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    id: i32,
    name: String,
    fedi_name: String,
    preferred_name: Option<String>,
    passhash: String,
    email: Option<String>,
    admin: bool,
    banned: bool,
    published: chrono::NaiveDateTime,
    updated: Option<chrono::NaiveDateTime>,
    theme: String,
    default_sort_type: i16,
    default_listing_type: i16,
    avatar: Option<String>,
    email_notifications_enabled: bool,
    show_nsfw: bool,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Clone, Default)]
#[diesel(table_name = user_)]
pub struct InsertUser {
    pub name: String,
    pub fedi_name: String,
    pub passhash: String,
    pub email: Option<String>,
}

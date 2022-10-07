use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::user_;
use chrono::NaiveDateTime;

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
    published: NaiveDateTime,
    updated: Option<NaiveDateTime>,
    theme: String,
    default_sort_type: i16,
    default_listing_type: i16,
    avatar: Option<String>,
    email_notifications_enabled: bool,
    show_nsfw: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_)]
pub struct UserForm {
    pub name: Option<String>,
    pub fedi_name: Option<String>,
    pub preferred_name: Option<Option<String>>,
    pub passhash: Option<String>,
    pub email: Option<String>,
    pub admin: Option<bool>,
    pub banned: Option<bool>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub theme: Option<String>,
    pub default_sort_type: Option<i16>,
    pub default_listing_type: Option<i16>,
    pub avatar: Option<Option<String>>,
    pub email_notifications_enabled: Option<bool>,
    pub show_nsfw: Option<bool>,
}
use crate::schema::user_;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub preferred_name: Option<String>,
    pub passhash: String,
    pub email: Option<String>,
    pub admin: bool,
    pub banned: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub avatar: Option<String>,
    pub email_notifications_enabled: bool,
    pub show_nsfw: bool,
    pub accepted_application: bool,
    pub deleted: bool,
    pub expires: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_)]
pub struct UserSafe {
    pub id: i32,
    pub name: String,
    pub preferred_name: Option<String>,
    pub admin: bool,
    pub banned: bool,
    pub published: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub avatar: Option<String>,
    pub email_notifications_enabled: bool,
    pub show_nsfw: bool,
    pub deleted: bool,
    pub expires: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
}

/// Struct for retrieving setting columns from user table
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = user_)]
pub struct UserSettings {
    pub id: i32,
    pub email: Option<String>,
    pub show_nsfw: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
}


#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_)]
pub struct UserForm {
    pub name: String,
    pub preferred_name: Option<Option<String>>,
    pub passhash: String,
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
    pub accepted_application: Option<bool>,
    pub deleted: Option<bool>,
    pub expires: Option<Option<NaiveDateTime>>,
    pub banner: Option<Option<String>>,
    pub bio: Option<Option<String>>,
}

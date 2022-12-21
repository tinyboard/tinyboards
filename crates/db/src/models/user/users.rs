use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub preferred_name: Option<String>,
    pub passhash: String,
    pub email: Option<String>,
    pub login_nonce: Option<i32>,
    pub is_admin: bool,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub avatar: Option<String>,
    pub email_notifications_enabled: bool,
    pub show_nsfw: bool,
    pub accepted_application: bool,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub is_application_accepted: bool,
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct UserSafe {
    pub id: i32,
    pub name: String,
    pub preferred_name: Option<String>,
    pub is_admin: bool,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub email_notifications_enabled: bool,
    pub show_nsfw: bool,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub is_application_accepted: bool,
}

/// Struct for retrieving setting columns from user table
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct UserSettings {
    pub id: i32,
    pub email: Option<String>,
    pub show_nsfw: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub bio: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserForm {
    pub name: String,
    pub preferred_name: Option<Option<String>>,
    pub passhash: String,
    pub email: Option<String>,
    pub login_nonce: Option<Option<i32>>,
    pub is_admin: Option<bool>,
    pub is_banned: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub theme: Option<String>,
    pub default_sort_type: Option<i16>,
    pub default_listing_type: Option<i16>,
    pub avatar: Option<Option<String>>,
    pub email_notifications_enabled: Option<bool>,
    pub show_nsfw: Option<bool>,
    pub accepted_application: Option<bool>,
    pub is_deleted: Option<bool>,
    pub unban_date: Option<Option<NaiveDateTime>>,
    pub banner: Option<Option<String>>,
    pub bio: Option<Option<String>>,
    pub is_application_accepted: Option<bool>,
}

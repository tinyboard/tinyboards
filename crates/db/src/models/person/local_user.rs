use crate::schema::local_user;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = local_user)]
pub struct LocalUser {
    pub id: i32,
    pub name: String,
    pub person_id: i32,
    pub passhash: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub is_banned: bool,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub lang: String,
    pub email_notifications_enabled: bool,
    pub accepted_application: bool,
    pub is_application_accepted: bool,
    pub email_verified: bool,
    pub updated: Option<NaiveDateTime>,
    pub creation_date: NaiveDateTime,
} 

/// Struct for retrieving setting columns from user table
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = local_user)]
pub struct LocalUserSettings {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
    pub lang: String,
    pub updated: Option<NaiveDateTime>,
}


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = local_user)]
pub struct LocalUserForm {
    pub name: Option<String>,
    pub person_id: Option<i32>,
    pub passhash: Option<String>,
    pub email: Option<Option<String>>,
    pub is_admin: Option<bool>,
    pub is_banned: Option<bool>,
    pub is_deleted: Option<bool>, 
    pub unban_date: Option<Option<NaiveDateTime>>,
    pub show_nsfw: Option<bool>,
    pub show_bots: Option<bool>,
    pub theme: Option<String>,
    pub default_sort_type: Option<i16>,
    pub default_listing_type: Option<i16>,
    pub lang: Option<String>,
    pub email_notifications_enabled: Option<bool>,
    pub accepted_application: Option<bool>,
    pub is_application_accepted: Option<bool>,
    pub email_verified: Option<bool>,
    pub updated: Option<Option<NaiveDateTime>>,
}


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable, Default)]
#[diesel(table_name = local_user)]
pub struct LocalUserSafe {
    pub id: i32,
    pub person_id: i32,
    pub name: String,
    pub is_admin: bool,
    pub is_banned: bool,
    pub is_deleted: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub unban_date: Option<NaiveDateTime>,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub is_application_accepted: bool,
}

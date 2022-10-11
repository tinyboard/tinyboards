use crate::schema::user_;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub fedi_name: String,
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
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "user_")]
pub struct UserSafe {
    pub id: i32,
    pub name: String,
    pub fedi_name: String,
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
}


#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_)]
pub struct UserForm {
    pub name: String,
    pub fedi_name: Option<String>,
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
}

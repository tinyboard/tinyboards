use crate::newtypes::DbUrl;
use crate::schema::person;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub is_banned: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<DbUrl>,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<DbUrl>,
    pub bio: Option<String>,
    pub signature: Option<DbUrl>,
    pub bot_account: bool,
    pub last_refreshed_date: NaiveDateTime,
    pub instance_id: i32,
    pub is_admin: bool,
    pub instance: Option<String>,
    pub admin_level: i32,
    pub profile_background: Option<DbUrl>,
    pub avatar_frame: Option<DbUrl>,
    pub bio_html: Option<String>,
    pub profile_music: Option<DbUrl>,
    pub profile_music_youtube: Option<String>,
    pub board_creation_approved: bool,
}

/// A safe representation of user, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = person)]
pub struct PersonSafe {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub is_banned: bool,
    pub local: bool,
    pub actor_id: DbUrl,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<DbUrl>,
    pub signature: Option<DbUrl>,
    pub is_deleted: bool,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<DbUrl>,
    pub bio: Option<String>,
    pub inbox_url: DbUrl,
    pub shared_inbox_url: Option<DbUrl>,
    pub bot_account: bool,
    pub last_refreshed_date: NaiveDateTime,
    pub is_admin: bool,
    pub instance: Option<String>,
    pub admin_level: i32,
    pub profile_background: Option<DbUrl>,
    pub avatar_frame: Option<DbUrl>,
    pub bio_html: Option<String>,
    pub profile_music: Option<DbUrl>,
    pub profile_music_youtube: Option<String>,
    pub board_creation_approved: bool,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Default,
    Insertable,
    AsChangeset,
    TypedBuilder,
)]
#[builder(field_defaults(default))]
#[diesel(table_name = person)]
pub struct PersonForm {
    #[builder(!default)]
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub is_banned: Option<bool>,
    pub is_admin: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub avatar: Option<DbUrl>,
    pub signature: Option<DbUrl>,
    pub is_deleted: Option<bool>,
    pub unban_date: Option<NaiveDateTime>,
    pub banner: Option<DbUrl>,
    pub bio: Option<String>,
    pub bot_account: Option<bool>,
    pub last_refreshed_date: Option<NaiveDateTime>,
    #[builder(!default)]
    pub instance_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub instance: Option<String>,
    pub admin_level: Option<i32>,
    pub profile_background: Option<Option<DbUrl>>,
    pub avatar_frame: Option<Option<DbUrl>>,
    pub bio_html: Option<String>,
    pub profile_music: Option<Option<DbUrl>>,
    pub profile_music_youtube: Option<Option<String>>,
    pub board_creation_approved: Option<bool>,
}

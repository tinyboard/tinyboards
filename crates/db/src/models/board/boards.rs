use crate::{newtypes::DbUrl, schema::boards};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = boards)]
pub struct Board {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub actor_id: DbUrl,
    pub local: bool,
    pub private_key: Option<String>,
    pub public_key: String,
    pub subscribers_url: DbUrl,
    pub inbox_url: DbUrl,
    pub shared_inbox_url: Option<DbUrl>,
    pub last_refreshed_date: NaiveDateTime,
    pub instance_id: i32,
    pub moderators_url: Option<DbUrl>,
    pub featured_url: Option<DbUrl>,
    pub icon: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub posting_restricted_to_mods: bool,
    pub is_removed: bool,
    pub ban_reason: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    pub sidebar: Option<String>,
    pub sidebar_html: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset)]
#[diesel(table_name = boards)]
pub struct BoardForm {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub is_deleted: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub is_hidden: Option<bool>,
    pub actor_id: Option<DbUrl>,
    pub local: Option<bool>,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub subscribers_url: Option<DbUrl>,
    pub inbox_url: Option<DbUrl>,
    pub shared_inbox_url: Option<Option<DbUrl>>,
    pub last_refreshed_date: Option<NaiveDateTime>,
    pub instance_id: Option<i32>,
    pub moderators_url: Option<DbUrl>,
    pub featured_url: Option<DbUrl>,
    pub icon: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub posting_restricted_to_mods: Option<bool>,
    pub is_removed: Option<bool>,
    pub ban_reason: Option<Option<String>>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub sidebar: Option<Option<String>>,
    pub sidebar_html: Option<Option<String>>,
}

/// A safe representation of board, without the sensitive info
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = boards)]
pub struct BoardSafe {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub icon: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub description: Option<String>,
    pub creation_date: chrono::NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: bool,
    pub is_removed: bool,
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub actor_id: DbUrl,
    pub subscribers_url: DbUrl,
    pub inbox_url: DbUrl,
    pub shared_inbox_url: Option<DbUrl>,
    pub moderators_url: Option<DbUrl>,
    pub featured_url: Option<DbUrl>,
    pub ban_reason: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    pub sidebar: Option<String>,
    pub sidebar_html: Option<String>,
}

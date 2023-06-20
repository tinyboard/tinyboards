use crate::schema::person;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::newtypes::DbUrl;
use typed_builder::TypedBuilder;
use serde_with::skip_serializing_none;

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
    pub actor_id: DbUrl,
    pub local: bool,
    pub private_key: Option<String>,
    pub public_key: String,
    pub inbox_url: DbUrl,
    pub shared_inbox_url: Option<DbUrl>,
    pub bot_account: bool,
    pub last_refreshed_date: NaiveDateTime,
    pub instance_id: i32,
    pub is_admin: bool,
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
}


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, Insertable, AsChangeset, TypedBuilder)]
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
    pub actor_id: Option<DbUrl>,
    pub local: Option<bool>,
    pub private_key: Option<String>,
    #[builder(!default)]
    pub public_key: Option<String>,
    pub inbox_url: Option<DbUrl>,
    pub shared_inbox_url: Option<DbUrl>,
    pub bot_account: Option<bool>,
    pub last_refreshed_date: Option<NaiveDateTime>,
    #[builder(!default)]
    pub instance_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
}
use crate::schema::site;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site)]
pub struct Site {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub enable_downvotes: bool,
    pub open_registration: bool,
    pub enable_nsfw: bool,
    pub require_application: bool,
    pub application_question: Option<String>,
    pub private_instance: bool,
    pub email_verification_required: bool,
    pub invite_only: bool,
    pub default_avatar: Option<String>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site)]
pub struct SiteForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub creator_id: Option<i32>,
    pub updated: Option<Option<NaiveDateTime>>,
    pub enable_downvotes: Option<bool>,
    pub open_registration: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub require_application: Option<bool>,
    pub application_question: Option<Option<String>>,
    pub private_instance: Option<bool>,
    pub email_verification_required: Option<bool>,
    pub invite_only: Option<bool>,
    pub default_avatar: Option<Option<String>>,
}

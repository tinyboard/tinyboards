use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::site_invite;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site_invite)]
pub struct SiteInvite {
    pub id: i32,
    pub verification_code: String,
    pub published: NaiveDateTime,
    pub validated: bool,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site_invite)]
pub struct SiteInviteForm {
    pub verification_code: Option<String>,
    pub validated: Option<bool>,
}
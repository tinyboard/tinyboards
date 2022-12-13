use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use crate::schema::site_invite;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = site_invite)]
pub struct SiteInvite {
    pub id: i32,
    pub email: String,
    pub verification_code: String,
    pub published: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = site_invite)]
pub struct SiteInviteForm {
    pub email: String,
    pub verification_code: String,
}
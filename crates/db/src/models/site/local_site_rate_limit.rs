use crate::schema::local_site_rate_limit;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = local_site_rate_limit)]
pub struct LocalSiteRateLimit {
    pub id: i32,
    pub local_site_id: i32,
    pub message: i32,
    pub message_per_second: i32,
    pub post: i32,
    pub post_per_second: i32,
    pub register: i32,
    pub register_per_second: i32,
    pub image: i32,
    pub image_per_second: i32,
    pub comment: i32,
    pub comment_per_second: i32,
    pub search: i32,
    pub search_per_second: i32,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = local_site_rate_limit)]
pub struct LocalSiteRateLimitForm {
    pub message: Option<i32>,
    pub message_per_second: Option<i32>,
    pub post: Option<i32>,
    pub post_per_second: Option<i32>,
    pub register: Option<i32>,
    pub register_per_second: Option<i32>,
    pub image: Option<i32>,
    pub image_per_second: Option<i32>,
    pub comment: Option<i32>,
    pub comment_per_second: Option<i32>,
    pub search: Option<i32>,
    pub search_per_second: Option<i32>,
    pub updated: Option<NaiveDateTime>,
}
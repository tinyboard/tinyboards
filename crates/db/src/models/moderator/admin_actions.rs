use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::schema::{
    admin_purge_board,
    admin_purge_comment,
    admin_purge_post,
    admin_purge_person,
};


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admin_purge_board)]
pub struct AdminPurgeBoard {
    pub id: i32,
    pub admin_id: i32,
    pub board_id: i32,
    pub reason: Option<String>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = admin_purge_board)]
pub struct AdminPurgeBoardForm {
    pub admin_id: i32,
    pub board_id: i32,
    pub reason: Option<Option<String>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admin_purge_comment)]
pub struct AdminPurgeComment {
    pub id: i32,
    pub admin_id: i32,
    pub comment_id: i32,
    pub reason: Option<String>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = admin_purge_comment)]
pub struct AdminPurgeCommentForm {
    pub admin_id: i32,
    pub comment_id: i32,
    pub reason: Option<Option<String>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admin_purge_post)]
pub struct AdminPurgePost {
    pub id: i32,
    pub admin_id: i32,
    pub post_id: i32,
    pub reason: Option<String>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = admin_purge_post)]
pub struct AdminPurgePostForm {
    pub admin_id: i32,
    pub post_id: i32,
    pub reason: Option<Option<String>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admin_purge_person)]
pub struct AdminPurgePerson {
    pub id: i32,
    pub admin_id: i32,
    pub person_id: i32,
    pub reason: Option<String>,
    pub when_: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = admin_purge_person)]
pub struct AdminPurgePersonForm {
    pub admin_id: i32,
    pub person_id: i32,
    pub reason: Option<Option<String>>,
}
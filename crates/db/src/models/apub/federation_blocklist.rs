use crate::schema::federation_blocklist;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = federation_blocklist)]
pub struct FederationBlockList {
    pub id: i32,
    pub instance_id: i32,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = federation_blocklist)]
pub struct FederationBlockListForm {
    pub instance_id: i32,
    pub updated: Option<NaiveDateTime>,
}
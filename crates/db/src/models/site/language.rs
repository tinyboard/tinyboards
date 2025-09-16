use crate::schema::language;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = language)]
pub struct Language {
    pub id: i32,
    pub code: String,
    pub name: String,
}
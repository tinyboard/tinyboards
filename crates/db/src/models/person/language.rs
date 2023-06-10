use crate::schema::language;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = language)]
pub struct LocalUserLanguage {
    pub id: i32,
    pub code: Option<String>,
    pub name: Option<String>,
}
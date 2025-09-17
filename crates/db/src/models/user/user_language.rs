use crate::schema::user_language;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = user_language)]
pub struct UserLanguage {
    pub id: i32,
    pub user_id: i32,
    pub language_id: i32,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = user_language)]
pub struct UserLanguageForm {
    pub user_id: i32,
    pub language_id: i32,
}
use crate::schema::local_user_language;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = local_user_language)]
pub struct LocalUserLanguage {
    pub id: i32,
    pub local_user_id: i32,
    pub language_id: i32,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = local_user_language)]
pub struct LocalUserLanguageForm {
    pub local_user_id: i32,
    pub language_id: i32,
}
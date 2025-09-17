use crate::schema::secret;
use diesel::prelude::*;

#[derive(Clone)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = secret)]
pub struct Secret {
    pub id: i32,
    pub jwt: String,
}
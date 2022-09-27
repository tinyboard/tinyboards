use crate::models::users::{Users, InsertUser};
use diesel::prelude::*;
use diesel::{PgConnection, insert_into};

impl Users {
    /**
     * load up to a `limit` amount of users. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::users::dsl::*;
        users
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load users")
    }
}

impl InsertUser {
    /**
     * insert a user into the database. Requires a **mutable** reference to a database connection.
     */
    pub fn insert(conn: &mut PgConnection, user_form: &InsertUser) -> QueryResult<usize> {
        use crate::schema::users::dsl::*;
        
        insert_into(users)
            .values(user_form)
            .execute(conn)
    }
}
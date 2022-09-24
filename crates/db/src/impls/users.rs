use crate::models::users::Users;
use diesel::prelude::*;
use diesel::PgConnection;

impl Users {
    /**
     * Load up to `limit` amount of users. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::users::dsl::*;

        // TODO: make this a (way) more complex query later
        // TODO: better error handling instead of panicking?
        posts
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load users")
    }
}

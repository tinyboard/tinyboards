use crate::models::post::Post;
use diesel::prelude::*;
use diesel::PgConnection;

impl Post {
    /**
     * Load up to `limit` amount of posts. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::posts::dsl::*;

        // TODO: make this a (way) more complex query later
        // TODO: better error handling instead of panicking?
        posts
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load posts")
    }
}

use crate::models::alts::Alts;
use diesel::prelude::*;
use diesel::PgConnection;

impl Alts {
    /**
     * Load up to the `limit` amount of posts. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::alts::dsl::*;
        alts
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load alts")
    }
}

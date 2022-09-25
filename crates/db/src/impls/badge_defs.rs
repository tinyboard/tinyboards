use crate::models::badge_defs::BadgeDefs;
use diesel::prelude::*;
use diesel::PgConnection;

impl BadgeDefs {
    /**
     * Load up to the `limit` amount of BadgeDefs. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::badge_defs::dsl::*;
        badge_defs
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load alts")
    }
}

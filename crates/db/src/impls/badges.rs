use crate::models::badges::Badges;
use diesel::prelude::*;
use diesel::PgConnection;

impl Badges {
    /**
     * Load up to the `limit` amount of Badges. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Vec<Self> {
        use crate::schema::badges::dsl::*;
        badges
            .limit(limit)
            .load::<Self>(conn)
            .expect("Failed to load alts")
    }
}

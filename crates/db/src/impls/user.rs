use crate::models::user::{InsertUser, User};
use diesel::prelude::*;
use diesel::PgConnection;

use porpl_utils::PorplError;

impl User {
    /**
     * load up to a `limit` amount of users. Requires a **mutable** reference to a database connection.
     */
    pub fn load(conn: &mut PgConnection, limit: i64) -> Result<Vec<Self>, PorplError> {
        use crate::schema::users::dsl::*;
        users.limit(limit).load::<Self>(conn).map_err(|e| {
            eprintln!("ERROR: {}", e.to_string());
            PorplError::new(500, String::from("Internal server error :\\"))
        })
    }

    pub fn insert(
        conn: &mut PgConnection,
        username: String,
        password: String,
    ) -> Result<Self, PorplError> {
        use crate::schema::users;

        let new_user = InsertUser {
            username,
            passhash: password,
            email: "".to_owned(),
            created_utc: 12,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e.to_string());
                PorplError::new(500, String::from("Internal Server Error"))
            })
    }
}

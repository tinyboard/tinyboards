use crate::models::comments::*;
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::PorplError;
use porpl_utils::time::utc_timestamp;

impl Comments {
    pub fn insert(
        conn: &mut PgConnection,
        author_id: i32,
        parent_submission: i32,
        body: String,
    ) -> Result<Self, PorplError> {
        use crate::schema::comments;

        let created_utc = utc_timestamp();

        let new_comment = InsertComment {
            author_id,
            parent_submission,
            body,
            created_utc,
        };

        diesel::insert_into(comments::table)
            .values(&new_comment)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {e}");
                PorplError::err_500()
            })
    }
}
use crate::models::submissions::*;
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::PorplError;

impl Submissions {
    pub fn insert(
        conn: &mut PgConnection,
        title: String,
        post_url: Option<String>,
        body: Option<String>,
        created_utc: i64,
        author_id: i32,
    ) -> Result<Self, PorplError> {
        use crate::schema::submissions;

        let new_submission = InsertSubmission {
            title,
            post_url,
            body,
            created_utc,
            author_id,
        };

        diesel::insert_into(submissions::table)
            .values(&new_submission)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
}

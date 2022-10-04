use crate::models::submissions::*;
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::PorplError;
use crate::traits::Crud;

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


impl Crud for Submissions {
    type Form = SubmissionForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, submission_id: i32) -> Result<Self, PorplError> {
        use crate::schema::submissions::dsl::*;
        submissions
            .find(submission_id)
            .first::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn delete(conn: &mut PgConnection, submission_id: i32) -> Result<usize, PorplError> {
        use crate::schema::submissions::dsl::*;
        diesel::delete(submissions.find(submission_id))
            .execute(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn create(conn: &mut PgConnection, form: &SubmissionForm) -> Result<Self, PorplError> {
        use crate::schema::submissions::dsl::*;
        diesel::insert_into(submissions)
            .values(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
    fn update(conn: &mut PgConnection, submission_id: i32, form: &SubmissionForm) -> Result<Self, PorplError> {
        use crate::schema::submissions::dsl::*;
        diesel::update(submissions.find(submission_id))
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }
}

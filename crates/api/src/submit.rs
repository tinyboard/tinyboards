use diesel::PgConnection;
// external crates
use regex::Regex;
use serde::{Deserialize, Serialize};

// internal crates
use crate::data::PorplContext;
use crate::utils::{blocking, require_user};
use crate::Perform;
use porpl_db::models::submissions::Submissions;
use porpl_utils::PorplError;


#[derive(Deserialize)]
pub struct CreateSubmission {
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>
}

#[derive(Serialize)]
pub struct CreateSubmissionResponse {
    pub message: String
}

fn validate_post_url(url: &String) -> Result<(), PorplError> {
    let re = Regex::new(r"((https)://)(www.)?[a-zA-Z0-9@:%._\\+~#?&//=]{2,256}\\.[a-z]{2,6}\\b([-a-zA-Z0-9@:%._\\+~#?&//=]*)").unwrap();
    if re.is_match(url) {
        Ok(())
    } else {
        Err(PorplError::new(400, String::from("Invalid post url!")))
    }
}

#[async_trait::async_trait]
impl Perform for CreateSubmission {
    type Response = CreateSubmissionResponse;

    async fn perform(self, context: &PorplContext, auth: Option<&str>) -> Result<Self::Response, PorplError> {
        
        let data = self;

        let u = require_user(context.pool(), context.master_key(), auth).await?;

        let uid = u.id;

        if Some(data.url) {
            validate_post_url(&data.url)?;
        }

        let tstamp = porpl_utils::time::utc_timestamp();

        let new_submission = blocking(context.pool(), move |conn| {
            Submissions::insert(conn, data.title, data.url, data.body, tstamp, uid)
        })
        .await??;

        Ok(CreateSubmissionResponse { message: String::from("Post submitted successfully!") })
    }
}



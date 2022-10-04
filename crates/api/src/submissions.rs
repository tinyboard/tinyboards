// external crates
use regex::Regex;
use serde::{Deserialize, Serialize};

// internal crates
use crate::data::PorplContext;
use crate::utils::{blocking, require_user, load_user_opt};
use crate::{Perform};
use porpl_db::models::{submissions::Submissions, users::User};
use porpl_utils::PorplError;


#[derive(Deserialize)]
pub struct GetPost {
    pub post_id: i32,
}

#[derive(Deserialize)]
pub struct CreateSubmission {
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,
}

#[derive(Serialize)]
pub struct CreateSubmissionResponse {
    pub message: String,
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

    async fn perform(
        self,
        context: &PorplContext,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        let data = self;

        let User { id: uid, .. } = require_user(context.pool(), context.master_key(), auth).await?;

        if let Some(ref url) = data.url {
            validate_post_url(url)?;
        }

        let tstamp = porpl_utils::time::utc_timestamp();

        let _new_submission = blocking(context.pool(), move |conn| {
            Submissions::insert(conn, data.title, data.url, data.body, tstamp, uid)
        })
        .await??;

        Ok(CreateSubmissionResponse {
            message: String::from("Post submitted successfully!"),
        })
    }
}

#[async_trait::async_trait]
impl Perform for GetPost {
    type Response = Submissions;

    async fn perform(
        self,
        context: &PorplContext,
        auth: Option<&str>
    ) -> Result<Self::Response, PorplError> {

        let data = self;

        if auth.is_some() {
            load_user_opt(context.pool(), context.master_key(), auth).await?;
        }

        let post = blocking(context.pool(), move |conn| {
            Submissions::get_post(conn, data.post_id)
        }).await??;

        Ok(post)
    }
}
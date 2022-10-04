// external crates
//use regex::Regex;
use serde::{Deserialize, Serialize};

// internal crates
use crate::data::PorplContext;
use crate::utils::{blocking, require_user};
use crate::{Perform};
use porpl_db::models::{comments::Comments, users::User};
use porpl_utils::PorplError;

#[derive(Deserialize)]
pub struct CreateComment {
    pub comment_body: String,
}

#[derive(Serialize)]
pub struct CreateCommentResponse {
    pub message: String,
}

impl Perform for CreateComment {
    type Response = CreateCommentResponse;

    async fn perform(
        self,
        context: &PorplContext,
        auth: Option<&str>
    ) -> Result<Self::Response, PorplError> {
        
    }
}
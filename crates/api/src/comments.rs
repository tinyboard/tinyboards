// external crates
//use regex::Regex;
use serde::{Deserialize, Serialize};

// internal crates
use crate::data::PorplContext;
use crate::utils::{blocking, require_user};
use crate::{Perform};
use porpl_db::models::{comments::Comments};
use porpl_utils::PorplError;

#[derive(Deserialize)]
pub struct CreateComment {
    pub comment_body: String,
    pub post_id: i32, //GOAL IS TO REMOVE THIS
}

#[derive(Serialize)]
pub struct CreateCommentResponse {
    pub message: String,
}

#[async_trait::async_trait]
impl Perform for CreateComment {
    type Response = CreateCommentResponse;

    async fn perform(
        self,
        context: &PorplContext,
        auth: Option<&str>
    ) -> Result<Self::Response, PorplError> {
        let data = self;

        let user = require_user(context.pool(), context.master_key(), auth).await?;

        let _new_comment = blocking(context.pool(), move |conn| {
            Comments::insert(conn, user.id, data.post_id, data.comment_body)
        }).await??;

        Ok(CreateCommentResponse { message: String::from("Comment was submitted successfully!") })
    }
}
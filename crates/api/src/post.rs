use crate::Perform;
use serde::{Deserialize, Serialize};

use crate::data::PorplContext;
use porpl_db::models::post::Post;
use porpl_utils::PorplError;

use crate::utils::blocking;

#[derive(Deserialize)]
pub struct GetPosts {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct GetPostsResponse {
    listing: Vec<Post>,
}

#[async_trait::async_trait]
impl Perform for GetPosts {
    type Response = GetPostsResponse;

    async fn perform(self, context: &PorplContext) -> Result<Self::Response, PorplError> {
        let data: &GetPosts = &self;

        let limit = data.limit.unwrap_or(25);

        let posts = blocking(context.pool(), move |conn| Post::load(conn, limit))
            .await?
            .map_err(|e| PorplError::new(e.0, e.1))?;

        Ok(GetPostsResponse { listing: posts })
    }
}

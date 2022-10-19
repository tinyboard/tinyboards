use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateComment {
    pub body: String,
    pub post_id: i32,
    pub parent_id: Option<i32>, // parent comment id
}

/*#[derive(Serialize)]
pub struct CreateCommentResponse {

}*/

#[derive(Deserialize)]
pub struct GetPostComments {}

#[derive(Deserialize)]
pub struct GetPostCommentsRoute {
    pub post_id: i32,
}

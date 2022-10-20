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

#[derive(Deserialize, Serialize)]
pub struct CreateCommentLike {
    pub comment_id: i32,
    pub score: i16,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CommentResponse {
//     pub comment_view: CommentView,
//     pub recipient_ids: Vec<i32>,
// }
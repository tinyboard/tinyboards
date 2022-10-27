use crate::sensitive::Sensitive;
use porpl_db_views::structs::{BoardModeratorView, BoardView, PostView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostResponse {
    pub post_view: PostView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostIdPath {
    pub post_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SubmitPost {
    pub title: String,
    pub type_: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub board_id: Option<i32>,
    pub nsfw: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitPostResponse {
    pub message: String,
    pub status_code: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPost {
    pub id: Option<i32>,
    pub comment_id: Option<i32>,
    pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostResponse {
    pub post_view: PostView,
    pub board_view: BoardView,
    pub moderators: Vec<BoardModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPosts {
    pub listing_type: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub saved_only: Option<bool>,
    pub auth: Option<Sensitive<String>>,
}

#[derive(Deserialize)]
pub struct GetPostComments {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPostsResponse {
    pub posts: Vec<PostView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePostVote {
    pub score: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SavePost {
    pub save: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditPost {
    pub body: Option<String>,
    pub body_html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeletePost {
    pub deleted: bool,
}

use porpl_db::{
    models::{
        user::user::User,
    }
};
use porpl_db_views::structs::CommentView;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateComment {
    pub body: String,
    pub post_id: i32,
    pub parent_id: Option<i32>, // parent comment id
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentIdPath {
    pub comment_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SaveComment {
    pub save: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeleteComment {
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditComment {
    pub body: Option<String>,
    pub body_html: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateCommentVote {
    pub score: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListComments {
    pub listing_type: Option<String>,
    pub sort: Option<String>,
    pub board_id: Option<i32>,
    pub post_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub user: Option<User>,
    pub search_term: Option<String>,
    pub saved_only: Option<bool>,
    pub show_deleted_and_removed: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<CommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentResponse {
    pub comment_view: CommentView
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComment {}


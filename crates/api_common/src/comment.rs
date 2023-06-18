use serde::{Deserialize, Serialize};
use tinyboards_db::models::person::person::Person;
use tinyboards_db_views::structs::CommentView;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateComment {
    pub body: String,
    pub post_id: i32,
    pub parent_id: Option<i32>, // parent comment id
    pub language_id: Option<i32>,
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
    pub body: String,
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
    pub person: Option<Person>,
    pub search_term: Option<String>,
    pub saved_only: Option<bool>,
    pub show_deleted_and_removed: Option<bool>,
    pub format: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<CommentView>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentResponse {
    pub comment_view: CommentView,
    pub recipient_ids: Vec<i32>,
    /// optional front end id to tell which is coming back
    pub form_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComment {
    pub context: Option<i32>,
    pub sort: Option<String>,
    pub post: Option<i32>,
}

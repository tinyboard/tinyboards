use serde::{Deserialize, Serialize};
use tinyboards_db::{models::person::person::Person, CommentSortType, ListingType};
use tinyboards_db_views::structs::{CommentReportView, CommentView};

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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComment {
    pub context: Option<i32>,
    pub sort: Option<String>,
    pub post: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Remove a comment (only doable by mods)
pub struct ToggleCommentRemove {
    pub target_id: i32,
    pub removed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Report a comment.
pub struct CreateCommentReport {
    pub comment_id: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The comment report response.
pub struct CommentReportResponse {
    pub comment_report_view: CommentReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Resolve a comment report (only doable by mods).
pub struct ResolveCommentReport {
    pub report_id: i32,
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// List comment reports.
pub struct ListCommentReports {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Only shows the unresolved reports
    pub unresolved_only: Option<bool>,
    /// if no board is given, it returns reports for all boards moderated by the auth user
    pub board_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// List comment reports for a given comment.
pub struct GetCommentReports {
    pub comment_id: i32,
    /// Only shows the unresolved reports
    pub unresolved_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The comment report list response.
pub struct ListCommentReportsResponse {
    pub reports: Vec<CommentReportView>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetComments {
    pub type_: Option<ListingType>,
    pub sort: Option<String>,
    pub max_depth: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub post_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub saved_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The comment list response.
pub struct GetCommentsResponse {
    pub comments: Vec<CommentView>,
    pub total_count: i64,
}

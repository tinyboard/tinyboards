use crate::sensitive::Sensitive;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tinyboards_db::{
    aggregates::structs::PersonAggregates, newtypes::DbUrl, ListingType,
    PostFeatureType, /* SortType, */
};
use tinyboards_db_views::structs::{BoardModeratorView, BoardView, PostReportView, PostView};

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
    pub url: Option<DbUrl>,
    pub image: Option<DbUrl>,
    pub body: Option<String>,
    pub board_id: Option<i32>,
    pub is_nsfw: bool,
    pub language_id: Option<i32>,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostResponse {
    pub post_view: PostView,
    pub board_view: BoardView,
    pub moderators: Vec<BoardModeratorView>,
    pub author_counts: PersonAggregates,
}

#[derive(Deserialize)]
pub struct GetPostComments {}

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
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeletePost {
    pub deleted: bool,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
/// Site metadata, from its opengraph tags.
pub struct SiteMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub(crate) image: Option<DbUrl>,
    pub embed_video_url: Option<DbUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Feature a post (stickies/pin to top)
pub struct FeaturePost {
    pub post_id: i32,
    pub featured: bool,
    pub feature_type: PostFeatureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Remove a post (only doable by mods).
pub struct RemovePost {
    pub target_id: i32,
    pub removed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Lock or unlock a post (only doable by mods).
pub struct LockPost {
    pub post_id: i32,
    pub locked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Create a post report.
pub struct CreatePostReport {
    pub post_id: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The post report response.
pub struct PostReportResponse {
    pub post_report_view: PostReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Resolve a post report (mods only).
pub struct ResolvePostReport {
    pub report_id: i32,
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// List post reports.
pub struct ListPostReports {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Only shows the unresolved reports
    pub unresolved_only: Option<bool>,
    /// if no board is given, it returns reports for all boards moderated by the auth user
    pub board_id: Option<i32>,
    pub auth: Sensitive<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// returns all reports for a given post
pub struct GetPostReports {
    pub post_id: i32,
    pub unresolved_only: bool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The post reports response.
pub struct ListPostReportsResponse {
    pub post_reports: Vec<PostReportView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
/// Get a list of posts.
pub struct GetPosts {
    pub type_: Option<ListingType>,
    //pub sort: Option<SortType>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub saved_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The post list response.
pub struct GetPostsResponse {
    pub posts: Vec<PostView>,
    pub total_count: i64,
}

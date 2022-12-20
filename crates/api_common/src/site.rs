use serde::{Deserialize, Serialize};
use tinyboards_db::SiteMode;
use tinyboards_db_views::{structs::{UserView, CommentView, PostView, BoardView, SiteInviteView}};
use tinyboards_db_views_mod::structs::{ModRemovePostView, ModLockPostView, ModRemoveCommentView, ModRemoveBoardView, ModBanFromBoardView, ModBanView, ModAddBoardModView, ModAddAdminView, ModStickyPostView, AdminPurgeUserView, AdminPurgeBoardView, AdminPurgePostView, AdminPurgeCommentView};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Search {
    pub query: Option<String>,
    pub domain: Option<String>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub creator_id: Option<i32>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub sort: Option<String>,
    pub listing_type: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SearchResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub comments: Vec<CommentView>,
    pub posts: Vec<PostView>,
    pub boards: Vec<BoardView>,
    pub users: Vec<UserView>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GetFeed {
    pub listing_type: Option<String>,
    pub sort: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub user_id: Option<i32>,
    pub search: Option<String>,
    pub saved_only: Option<bool>,
    pub nsfw: Option<bool>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembers {
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembersResponse {
    pub members: Vec<UserView>,
    pub total_count: i64,
}

#[derive(Serialize)]
pub struct Message {
    pub code: i32,
    pub message: String,
}

/// Generic response
impl Message {
    pub fn new(msg: &str) -> Self {
        Self {
            code: 200,
            message: String::from(msg),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetModLog {
    pub mod_user_id: Option<i32>,
    pub board_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub other_user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetModLogResponse {
    pub removed_posts: Vec<ModRemovePostView>,
    pub locked_posts: Vec<ModLockPostView>,
    pub stickied_posts: Vec<ModStickyPostView>,
    pub removed_comments: Vec<ModRemoveCommentView>,
    pub removed_boards: Vec<ModRemoveBoardView>,
    pub banned_from_board: Vec<ModBanFromBoardView>,
    pub banned_from_site: Vec<ModBanView>,
    pub mods_added: Vec<ModAddBoardModView>,
    pub admins_added: Vec<ModAddAdminView>,
    pub admin_purged_users: Vec<AdminPurgeUserView>,
    pub admin_purged_boards: Vec<AdminPurgeBoardView>,
    pub admin_purged_posts: Vec<AdminPurgePostView>,
    pub admin_purged_comments: Vec<AdminPurgeCommentView>,   
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetSiteSettings {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteSettingsResponse {
    pub name: String,
    pub description: String,
    pub site_mode: SiteMode,
    pub enable_downvotes: bool,
    pub enable_nsfw: bool,
    pub application_question: String,
    pub private_instance:  bool,
    pub email_verification_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveSiteSettings {
    pub name: Option<String>,
    pub description: Option<String>,
    pub site_mode: Option<SiteMode>,
    pub enable_downvotes: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub application_question: Option<String>,
    pub private_instance:  Option<bool>,
    pub email_verification_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSiteInvite {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSiteInviteResponse {
    pub invite_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListSiteInvites {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListSiteInvitesResponse {
    pub invites: Vec<SiteInviteView>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteSiteInvite {
    pub invite_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidateSiteInvite {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteToken {
    pub invite_token: String,
}
use serde::{Deserialize, Serialize};
use tinyboards_db::{SiteMode, newtypes::DbUrl, models::apub::{instance::Instance, language::Language}};
use tinyboards_db_views::structs::{BoardView, CommentView, PostView, SiteInviteView, PersonView, SiteView};
use tinyboards_db_views_mod::structs::{
    AdminPurgeBoardView, AdminPurgeCommentView, AdminPurgePostView, AdminPurgePersonView,
    ModAddAdminView, ModAddBoardModView, ModBanFromBoardView, ModBanView, ModLockPostView,
    ModRemoveBoardView, ModRemoveCommentView, ModRemovePostView, ModFeaturePostView,
};

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
    pub users: Vec<PersonView>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembers {
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
    pub is_admin: Option<bool>,
    pub is_banned: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetMembersResponse {
    pub members: Vec<PersonView>,
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
    pub mod_person_id: Option<i32>,
    pub board_id: Option<i32>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub other_person_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetModLogResponse {
    pub removed_posts: Vec<ModRemovePostView>,
    pub locked_posts: Vec<ModLockPostView>,
    pub featured_posts: Vec<ModFeaturePostView>,
    pub removed_comments: Vec<ModRemoveCommentView>,
    pub removed_boards: Vec<ModRemoveBoardView>,
    pub banned_from_board: Vec<ModBanFromBoardView>,
    pub banned_from_site: Vec<ModBanView>,
    pub mods_added: Vec<ModAddBoardModView>,
    pub admins_added: Vec<ModAddAdminView>,
    pub admin_purged_persons: Vec<AdminPurgePersonView>,
    pub admin_purged_boards: Vec<AdminPurgeBoardView>,
    pub admin_purged_posts: Vec<AdminPurgePostView>,
    pub admin_purged_comments: Vec<AdminPurgeCommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetSiteSettings {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSite {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteResponse {
    pub site_view: SiteView,
    //pub taglines: Vec<TagLine>,
}
  
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteResponse {
    pub site_view: SiteView,
    pub admins: Vec<PersonView>,
    pub version: String,
    pub all_languages: Vec<Language>,
    pub discussion_languages: Vec<i32>,
    // list of taglines shown at the top of the front page
    // pub taglines: Vec<TagLine>,
    // list of custom emojis your site supports
    // pub custom_emojis: Vec<CustomEmojiView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSiteSettingsResponse {
    pub site_mode: SiteMode,
    pub name: String,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub enable_downvotes: bool,
    pub enable_nsfw: bool,
    pub application_question: String,
    pub private_instance: bool,
    pub require_email_verification: bool,
    pub default_avatar: String,
    pub welcome_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveSiteSettings {
    pub name: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub site_mode: Option<SiteMode>,
    pub enable_downvotes: Option<bool>,
    pub enable_nsfw: Option<bool>,
    pub application_question: Option<String>,
    pub private_instance: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub default_avatar: Option<String>,
    pub welcome_message: Option<String>,
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
pub struct DeleteSiteInvite {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteId {
    pub invite_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidateSiteInvite {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteToken {
    pub invite_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetTokenPath {
    pub reset_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutePasswordReset {
    pub new_password: String,
    pub new_password_verify: String,
} 

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutePasswordResetResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadResponse {
    pub uploads: Vec<DbUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetFile{
    pub thumbnail: Option<u32>,
    pub blur: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteFile{}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileNamePath {
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FederatedInstances {
    pub linked: Vec<Instance>,
    pub allowed: Vec<Instance>,
    pub blocked: Vec<Instance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetFederatedInstances {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetFederatedInstancesResponse {
    /// Optional if federation is disabled
    pub federated_instances: Option<FederatedInstances>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@tinyboards.net
  pub q: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FederatedObject {
    Comment(Option<CommentView>),
    Post(Option<PostView>),
    Board(Option<BoardView>),
    Person(Option<PersonView>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveObjectResponse {
    pub object: FederatedObject
}
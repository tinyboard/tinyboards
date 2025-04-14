use serde::{Deserialize, Serialize};
use tinyboards_db::{
    models::apub::{instance::Instance, language::Language},
    newtypes::DbUrl,
    SiteMode,
};
/*use tinyboards_db_views::structs::{
    BoardView, CommentView, PersonView, PostView, SiteInviteView, SiteView,
};*/
/*use tinyboards_db_views_mod::structs::{
    AdminPurgeBoardView, AdminPurgeCommentView, AdminPurgePersonView, AdminPurgePostView,
    ModAddAdminView, ModAddBoardModView, ModBanFromBoardView, ModBanView, ModFeaturePostView,
    ModLockPostView, ModRemoveBoardView, ModRemoveCommentView, ModRemovePostView,
};*/

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadResponse {
    pub uploads: Vec<DbUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetFile {
    pub thumbnail: Option<u32>,
    pub blur: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteFile {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileNamePath {
    pub file_name: String,
}

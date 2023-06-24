// use tinyboards_db::{
//     ListingType,
//     SortType,
// };
use serde::{Deserialize, Serialize};
use tinyboards_db::models::site::site::Site;
use tinyboards_db_views::structs::{BoardView, BoardModeratorView};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateBoard {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardResponse {
    pub board_view: BoardView,
    pub discussion_languages: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardIdPath {
    pub board_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditBoard {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_nsfw: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteBoard {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Remove a board (only doable by moderators).
pub struct RemoveBoard {
  pub board_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Hide a board from the main view
pub struct HideBoard {
    pub board_id: i32,
    pub hidden: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Subscribe to a board
pub struct SubscribeToBoard {
    pub board_id: i32,
    pub subscribe: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Block a board
pub struct BlockBoard {
    pub board_id: i32,
    pub block: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockBoardResponse {
    pub board_view: BoardView,
    pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddBoardMod {
    pub added: bool,
    pub person_id: i32,
    pub board_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddBoardModResponse {
    pub moderators: Vec<BoardModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
/// Get a board. Must provide either an id, or a name.
pub struct GetBoard {
  pub id: Option<i32>,
  /// Example: campfire, or campfire@xyz.tld
  pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// The board response.
pub struct GetBoardResponse {
  pub board_view: BoardView,
  pub site: Option<Site>,
  pub moderators: Vec<BoardModeratorView>,
  pub discussion_languages: Vec<i32>,
}

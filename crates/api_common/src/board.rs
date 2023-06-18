// use tinyboards_db::{
//     ListingType,
//     SortType,
// };
use serde::{Deserialize, Serialize};
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
pub struct AddModToBoard {
    pub board_id: i32,
    pub person_id: i32,
    pub added: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AddModToBoardResponse {
    pub moderators: Vec<BoardModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Subscribe to a board
pub struct SubscribeToBoard {
    pub board_id: i32,
    pub follow: bool,
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
// use tinyboards_db::{
//     ListingType,
//     SortType,
// };
use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::BoardView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateBoard {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardResponse {
    pub board_view: BoardView,
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

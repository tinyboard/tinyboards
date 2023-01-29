// use crate::sensitive::Sensitive;
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
pub struct CreateBoardResponse {
    pub board_view: BoardView,
}
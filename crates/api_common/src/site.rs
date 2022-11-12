use crate::sensitive::Sensitive;
use tinyboards_db_views::structs::{BoardModeratorView, BoardView, PostView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GetFeed {
    pub listing_type: Option<String>,
    pub sort: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub user_id: Option<i32>,
    pub saved_only: Option<bool>,
}

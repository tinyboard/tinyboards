use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::UserView;

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
}
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgePost {
    pub post_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeComment {

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeBoard {

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurgeUser {
    pub user_id: i32,
    pub reason: Option<String>,
}
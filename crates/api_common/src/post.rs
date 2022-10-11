use serde::{Deserialize, Serialize};
use porpl_db::{
    ListingType,
    SortType,
};
//use porpl_db_views::
use crate::sensitive::Sensitive;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SubmitPost {
    pub title: String,
    pub type_: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub creator_id: i32,
    pub board_id: i32,
    pub nsfw: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitPostResponse {
    pub message: String,
    pub status_code: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPosts {
    pub type_: Option<ListingType>,
    pub sort: Option<SortType>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub board_id: Option<i32>,
    pub board_name: Option<String>,
    pub saved_only: Option<bool>,
    pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListPostsResponse {
    pub posts: Vec<PostView>,
}

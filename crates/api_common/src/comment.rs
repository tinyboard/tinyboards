use porpl_db::{
    ListingType, 
    CommentSortType,
    models::{
        user::user::User,
    }
};
use porpl_db_views::structs::CommentView;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateComment {
    pub body: String,
    pub post_id: i32,
    pub parent_id: Option<i32>, // parent comment id
}

/*#[derive(Serialize)]
pub struct CreateCommentResponse {

}*/

#[derive(Deserialize)]
pub struct GetPostComments {}

#[derive(Deserialize)]
pub struct GetPostCommentsRoute {
    pub post_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct CreateCommentLike {
    pub comment_id: i32,
    pub score: i16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ListComments {
    pub listing_type: Option<ListingType>,
    pub sort: Option<CommentSortType>,
    pub board_id: Option<i32>,
    pub post_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub user: Option<User>,
    pub search_term: Option<String>,
    pub saved_only: Option<bool>,
    pub show_deleted_and_removed: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<CommentView>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CommentResponse {
//     pub comment_view: CommentView,
//     pub recipient_ids: Vec<i32>,
// }
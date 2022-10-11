use serde::{Serialize, Deserialize};

use crate::schema::{
    comment_aggregates,
    board_aggregates,
    user_aggregates,
    post_aggregates,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = comment_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::models::comment::comment::Comment)))]
pub struct CommentAggregates {
  pub id: i32,
  pub comment_id: CommentId,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: chrono::NaiveDateTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = board_aggregates))]
#[cfg_attr(
  feature = "full", diesel(belongs_to(crate::models::board::board::Board)))]
pub struct BoardAggregates {
  pub id: i32,
  pub board_id: CommunityId,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  pub published: chrono::NaiveDateTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = post_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::models::post::post::Post)))]
pub struct PostAggregates {
  pub id: i32,
  pub post_id: PostId,
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub stickied: bool,
  pub published: chrono::NaiveDateTime,
  pub newest_comment_time: chrono::NaiveDateTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = user_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::models::user::user::User)))]
pub struct UserAggregates {
    pub id: i32,
    pub user_id: i32,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
}
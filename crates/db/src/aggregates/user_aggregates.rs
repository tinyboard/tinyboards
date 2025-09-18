use crate::{schema::user_aggregates, utils::{get_conn, DbPool}};
use diesel::{result::Error, ExpressionMethods, QueryDsl, Queryable, Associations, Identifiable};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Queryable, Associations, Identifiable,
)]
#[diesel(belongs_to(crate::models::user::User, foreign_key = user_id))]
#[diesel(table_name = user_aggregates)]
pub struct UserAggregates {
    pub id: i32,
    pub user_id: i32,
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
}

impl UserAggregates {
    pub async fn read(pool: &DbPool, user_id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_aggregates::table
            .filter(user_aggregates::user_id.eq(user_id_))
            .first::<Self>(conn)
            .await
    }
}
use crate::{
        aggregates::structs::PostAggregates, 
        schema::post_aggregates
};
use diesel::{result::Error, *};

impl PostAggregates {
  pub fn read(conn: &mut PgConnection, post_id: PostId) -> Result<Self, Error> {
    post_aggregates::table
      .filter(post_aggregates::post_id.eq(post_id))
      .first::<Self>(conn)
  }
}
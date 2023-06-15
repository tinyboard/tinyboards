use crate::structs::BoardSubscriberView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
  schema::{boards, board_subscriptions, person},
  models::{board::boards::Board, person::person::Person},
  traits::JoinView,
  utils::{get_conn, DbPool},
};

type BoardSubscriberViewTuple = (Board, Person);

impl BoardSubscriberView {
  pub async fn for_board(pool: &DbPool, board_id: i32) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = board_subscriptions::table
      .inner_join(boards::table)
      .inner_join(person::table)
      .select((boards::all_columns, person::all_columns))
      .filter(board_subscriptions::board_id.eq(board_id))
      .order_by(boards::title)
      .load::<BoardSubscriberViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }

  pub async fn for_person(pool: &DbPool, person_id: i32) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = board_subscriptions::table
      .inner_join(boards::table)
      .inner_join(person::table)
      .select((boards::all_columns, person::all_columns))
      .filter(board_subscriptions::person_id.eq(person_id))
      .filter(boards::is_deleted.eq(false))
      .order_by(boards::title)
      .load::<BoardSubscriberViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for BoardSubscriberView {
  type JoinTuple = BoardSubscriberViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      board: a.0,
      subscriber: a.1,
    }
  }
}
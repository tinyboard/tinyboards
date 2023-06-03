use crate::structs::BoardModeratorView;
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, person::person::PersonSafe},
    schema::{board_mods, boards, person},
    traits::{ToSafe, ViewToVec}, utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;

type BoardModeratorViewTuple = (BoardSafe, PersonSafe);

impl BoardModeratorView {
    pub async fn for_board(pool: &DbPool, board_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let res = board_mods::table
            .inner_join(boards::table)
            .inner_join(person::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                PersonSafe::safe_columns_tuple(),
            ))
            .filter(board_mods::board_id.eq(board_id))
            .order_by(board_mods::creation_date)
            .load::<BoardModeratorViewTuple>(conn)
            .await?;

        Ok(Self::from_tuple_to_vec(res))
    }

    pub async fn for_user(pool: &DbPool, person_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let res = board_mods::table
            .inner_join(boards::table)
            .inner_join(person::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                PersonSafe::safe_columns_tuple(),
            ))
            .filter(board_mods::person_id.eq(person_id))
            .filter(boards::is_deleted.eq(false))
            .filter(boards::is_banned.eq(false))
            .order_by(board_mods::creation_date)
            .load::<BoardModeratorViewTuple>(conn)
            .await?;

        Ok(Self::from_tuple_to_vec(res))
    }
}

impl ViewToVec for BoardModeratorView {
    type DbTuple = BoardModeratorViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                board: a.0,
                moderator: a.1,
            })
            .collect::<Vec<Self>>()
    }
}

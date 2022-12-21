use crate::structs::BoardModeratorView;
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, user::users::UserSafe},
    schema::{board_mods, boards, users},
    traits::{ToSafe, ViewToVec},
};

type BoardModeratorViewTuple = (BoardSafe, UserSafe);

impl BoardModeratorView {
    pub fn for_board(conn: &mut PgConnection, board_id: i32) -> Result<Vec<Self>, Error> {
        let res = board_mods::table
            .inner_join(boards::table)
            .inner_join(users::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_mods::board_id.eq(board_id))
            .order_by(board_mods::creation_date)
            .load::<BoardModeratorViewTuple>(conn)?;

        Ok(Self::from_tuple_to_vec(res))
    }

    pub fn for_user(conn: &mut PgConnection, user_id: i32) -> Result<Vec<Self>, Error> {
        let res = board_mods::table
            .inner_join(boards::table)
            .inner_join(users::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_mods::user_id.eq(user_id))
            .filter(boards::is_deleted.eq(false))
            .filter(boards::is_banned.eq(false))
            .order_by(board_mods::creation_date)
            .load::<BoardModeratorViewTuple>(conn)?;

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

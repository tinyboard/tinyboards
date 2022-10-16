use crate::structs::BoardModeratorView;
use diesel::{result::Error, *};
use porpl_db::{
    schema::{
        board,
        board_moderator,
        user_,
    },
    models::{
        board::board::BoardSafe,
        user::user::UserSafe,
    },
    traits::{ToSafe, ViewToVec},
};

type BoardModeratorViewTuple = (
    BoardSafe,
    UserSafe
);

impl BoardModeratorView {
    pub fn for_board(
        conn: &mut PgConnection,
        board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let res = board_moderator::table
            .inner_join(board::table)
            .inner_join(user_::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_moderator::board_id.eq(board_id))
            .order_by(board_moderator::published)
            .load::<BoardModeratorViewTuple>(conn)?;

            Ok(Self::from_tuple_to_vec(res))
    }

    pub fn for_user(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let res = board_moderator::table
            .inner_join(board::table)
            .inner_join(user_::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_moderator::user_id.eq(user_id))
            .filter(board::deleted.eq(false))
            .filter(board::removed.eq(false))
            .order_by(board_moderator::published)
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

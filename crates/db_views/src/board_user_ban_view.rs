use crate::structs::BoardUserBanView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, user::user::UserSafe},
    schema::{board, board_user_ban, user_},
    traits::ToSafe,
};

type BoardUserBanViewTuple = (BoardSafe, UserSafe);

impl BoardUserBanView {
    pub fn get(
        conn: &mut PgConnection,
        from_user_id: i32,
        from_board_id: i32,
    ) -> Result<Self, Error> {
        let (board, user) = board_user_ban::table
            .inner_join(board::table)
            .inner_join(user_::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_user_ban::board_id.eq(from_board_id))
            .filter(board_user_ban::user_id.eq(from_user_id))
            .filter(
                board_user_ban::expires
                    .is_null()
                    .or(board_user_ban::expires.gt(now)),
            )
            .order_by(board_user_ban::published)
            .first::<BoardUserBanViewTuple>(conn)?;

        Ok(BoardUserBanView { board, user })
    }
}

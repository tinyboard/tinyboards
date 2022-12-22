use crate::structs::BoardUserBanView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, user::users::UserSafe},
    schema::{board_user_bans, boards, users},
    traits::ToSafe,
};

type BoardUserBanViewTuple = (BoardSafe, UserSafe);

impl BoardUserBanView {
    pub fn get(
        conn: &mut PgConnection,
        from_user_id: i32,
        from_board_id: i32,
    ) -> Result<Self, Error> {
        let (board, user) = board_user_bans::table
            .inner_join(boards::table)
            .inner_join(users::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_user_bans::board_id.eq(from_board_id))
            .filter(board_user_bans::user_id.eq(from_user_id))
            .filter(
                board_user_bans::expires
                    .is_null()
                    .or(board_user_bans::expires.gt(now)),
            )
            .order_by(board_user_bans::creation_date)
            .first::<BoardUserBanViewTuple>(conn)?;

        Ok(BoardUserBanView { board, user })
    }
}

use crate::structs::BoardUserBanView;
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    models::{board::boards::BoardSafe, local_user::users::UserSafe},
    schema::{board_user_bans, boards, users},
    traits::ToSafe, utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;

type BoardUserBanViewTuple = (BoardSafe, UserSafe);

impl BoardUserBanView {
    pub async fn get(
        pool: &DbPool,
        from_person_id: i32,
        from_board_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (board, user) = board_user_bans::table
            .inner_join(boards::table)
            .inner_join(users::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .filter(board_user_bans::board_id.eq(from_board_id))
            .filter(board_user_bans::person_id.eq(from_person_id))
            .filter(
                board_user_bans::expires
                    .is_null()
                    .or(board_user_bans::expires.gt(now)),
            )
            .order_by(board_user_bans::creation_date)
            .first::<BoardUserBanViewTuple>(conn)
            .await?;

        Ok(BoardUserBanView { board, user })
    }
}

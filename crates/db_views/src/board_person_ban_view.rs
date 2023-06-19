use crate::structs::BoardPersonBanView;
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{board::boards::BoardSafe, person::person::PersonSafe},
    schema::{board_person_bans, boards, person},
    traits::ToSafe,
    utils::{get_conn, DbPool},
};

type BoardPersonBanViewTuple = (BoardSafe, PersonSafe);

impl BoardPersonBanView {
    pub async fn get(
        pool: &DbPool,
        from_person_id: i32,
        from_board_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let ban = board_person_bans::table
            .inner_join(boards::table)
            .inner_join(person::table)
            .select((
                BoardSafe::safe_columns_tuple(),
                PersonSafe::safe_columns_tuple(),
            ))
            .filter(board_person_bans::board_id.eq(from_board_id))
            .filter(board_person_bans::person_id.eq(from_person_id))
            .filter(
                board_person_bans::expires
                    .is_null()
                    .or(board_person_bans::expires.gt(now)),
            )
            //.order_by(board_person_bans::creation_date)
            .first::<BoardPersonBanViewTuple>(conn)
            .await
            .optional()?
            .map(|(board, user)| BoardPersonBanView { board, user });

        Ok(ban)
    }
}

use crate::schema::{board_mods, board_user_bans, boards::dsl::*};
use crate::utils::naive_now;
use crate::{
    models::board::board_user_bans::{BoardUserBan, BoardUserBanForm},
    models::board::boards::{Board, BoardForm},
    traits::{Bannable, Crud},
};
use diesel::{dsl::*, prelude::*, result::Error, PgConnection, QueryDsl, RunQueryDsl};

impl Board {
    /// Takes a board id and an user id, and returns true if the user mods the board with the given id or is an admin
    pub fn board_has_mod(
        conn: &mut PgConnection,
        board_id: i32,
        user_id: i32,
    ) -> Result<bool, Error> {
        let mod_id = board_mods::table
            .select(board_mods::id)
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::user_id.eq(user_id))
            .first::<i32>(conn)
            .optional();

        mod_id.map(|opt| opt.is_some())
    }

    /// Takes a board id and an user id, and returns true if the user is banned from the board with the given id
    pub fn board_has_ban(
        conn: &mut PgConnection,
        board_id: i32,
        user_id: i32,
    ) -> Result<bool, Error> {
        let ban_id = board_user_bans::table
            .select(board_user_bans::id)
            .filter(board_user_bans::board_id.eq(board_id))
            .filter(board_user_bans::user_id.eq(user_id))
            .filter(
                board_user_bans::expires
                    .is_null()
                    .or(board_user_bans::expires.gt(now)),
            )
            .first::<i32>(conn)
            .optional();

        ban_id.map(|opt| opt.is_some())
    }

    pub fn update_banned(
        conn: &mut PgConnection,
        board_id: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_banned.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }
}

pub mod safe_type {
    use crate::{models::board::boards::BoardSafe, schema::boards::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        description,
        creation_date,
        updated,
        is_deleted,
        is_nsfw,
        is_hidden,
    );

    impl ToSafe for BoardSafe {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                description,
                creation_date,
                updated,
                is_deleted,
                is_nsfw,
                is_hidden,
            )
        }
    }
}

impl Crud for Board {
    type Form = BoardForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, board_id: i32) -> Result<Self, Error> {
        boards.find(board_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, board_id: i32) -> Result<usize, Error> {
        diesel::delete(boards.find(board_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &BoardForm) -> Result<Self, Error> {
        let new_board = diesel::insert_into(boards)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(new_board)
    }
    fn update(conn: &mut PgConnection, board_id: i32, form: &BoardForm) -> Result<Self, Error> {
        diesel::update(boards.find(board_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Bannable for BoardUserBan {
    type Form = BoardUserBanForm;

    fn ban(conn: &mut PgConnection, ban_form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::board_user_bans::dsl::{board_id, board_user_bans, user_id};
        insert_into(board_user_bans)
            .values(ban_form)
            .on_conflict((board_id, user_id))
            .do_update()
            .set(ban_form)
            .get_result::<Self>(conn)
    }

    fn unban(conn: &mut PgConnection, ban_form: &Self::Form) -> Result<usize, Error> {
        use crate::schema::board_user_bans::dsl::{board_id, board_user_bans, user_id};
        diesel::delete(
            board_user_bans
                .filter(board_id.eq(ban_form.board_id))
                .filter(user_id.eq(ban_form.user_id)),
        )
        .execute(conn)
    }
}

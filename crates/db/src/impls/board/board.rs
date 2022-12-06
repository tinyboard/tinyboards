use crate::schema::{board::dsl::*, board_moderator, board_user_ban};
use crate::utils::naive_now;
use crate::{
    models::board::board::{Board, BoardForm},
    models::board::board_user_ban::{BoardUserBan, BoardUserBanForm},
    traits::{Crud, Bannable},
};
use diesel::{dsl::*, prelude::*, result::Error, PgConnection, QueryDsl, RunQueryDsl};
use tinyboards_utils::TinyBoardsError;

impl Board {
    /// Takes a board id and an user id, and returns true if the user mods the board with the given id or is an admin
    pub fn board_has_mod(
        conn: &mut PgConnection,
        board_id: i32,
        user_id: i32,
    ) -> Result<bool, Error> {
        let mod_id = board_moderator::table
            .select(board_moderator::id)
            .filter(board_moderator::board_id.eq(board_id))
            .filter(board_moderator::user_id.eq(user_id))
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
        let ban_id = board_user_ban::table
            .select(board_user_ban::id)
            .filter(board_user_ban::board_id.eq(board_id))
            .filter(board_user_ban::user_id.eq(user_id))
            .filter(
                board_user_ban::expires
                    .is_null()
                    .or(board_user_ban::expires.gt(now)),
            )
            .first::<i32>(conn)
            .optional();

        ban_id.map(|opt| opt.is_some())
    }

    pub fn update_removed(
        conn: &mut PgConnection,
        board_id: i32,
        new_removed: bool,
    ) -> Result<Self, Error> {
        use crate::schema::board::dsl::*;
        diesel::update(board.find(board_id))
            .set((removed.eq(new_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }
}

pub mod safe_type {
    use crate::{models::board::board::BoardSafe, schema::board::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        description,
        published,
        updated,
        deleted,
        nsfw,
        hidden,
    );

    impl ToSafe for BoardSafe {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                description,
                published,
                updated,
                deleted,
                nsfw,
                hidden,
            )
        }
    }
}

impl Crud for Board {
    type Form = BoardForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, board_id: i32) -> Result<Self, Error> {
        board.find(board_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, board_id: i32) -> Result<usize, Error> {
        diesel::delete(board.find(board_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &BoardForm) -> Result<Self, Error> {
        let new_board = diesel::insert_into(board)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(new_board)
    }
    fn update(conn: &mut PgConnection, board_id: i32, form: &BoardForm) -> Result<Self, Error> {
        diesel::update(board.find(board_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Bannable for BoardUserBan {
    type Form = BoardUserBanForm;
    
    fn ban(
        conn: &mut PgConnection,
        ban_form: &Self::Form,
    ) -> Result<Self, Error> {
        use crate::schema::board_user_ban::dsl::{board_id, board_user_ban, user_id};
        insert_into(board_user_ban)
            .values(ban_form)
            .on_conflict((board_id, user_id))
            .do_update()
            .set(ban_form)
            .get_result::<Self>(conn)
    }

    fn unban(
        conn: &mut PgConnection,
        ban_form: &Self::Form,
    ) -> Result<usize, Error> {
        use crate::schema::board_user_ban::dsl::{board_id, board_user_ban, user_id};
        diesel::delete(
            board_user_ban
                .filter(board_id.eq(ban_form.board_id))
                .filter(user_id.eq(ban_form.user_id)),
        )
        .execute(conn)
    }
}
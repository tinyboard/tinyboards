use crate::schema::{board::dsl::*, board_moderator, board_user_ban};
use crate::{
    models::board::board::{Board, BoardForm},
    traits::Crud,
};
use diesel::{dsl::*, prelude::*, result::Error, PgConnection, QueryDsl, RunQueryDsl};

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

        mod_id.map(|opt| match opt {
            Some(_) => true,
            None => false,
        })
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

        ban_id.map(|opt| match opt {
            Some(_) => true,
            None => false,
        })
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

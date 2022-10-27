use crate::schema::board::dsl::*;
use crate::{
    models::board::board::{Board, BoardForm},
    traits::Crud,
};
use diesel::{result::Error, PgConnection, QueryDsl, RunQueryDsl};

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

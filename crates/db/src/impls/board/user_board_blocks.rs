use crate::schema::user_board_blocks::dsl::*;
use crate::{
    models::board::user_board_blocks::{BoardBlock, BoardBlockForm},
    traits::Blockable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;

impl BoardBlock {
    pub fn read(
        conn: &mut PgConnection,
        for_user_id: i32,
        for_board_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        user_board_blocks
            .filter(user_id.eq(for_user_id))
            .filter(board_id.eq(for_board_id))
            .first::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, "error reading board block"))
    }
}

impl Blockable for BoardBlock {
    type Form = BoardBlockForm;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(user_board_blocks)
            .values(form)
            .on_conflict((user_id, board_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, "could not block board"))
    }

    fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        diesel::delete(
            user_board_blocks
                .filter(user_id.eq(form.user_id))
                .filter(board_id.eq(form.board_id)),
        )
        .execute(conn)
        .map_err(|e| TinyBoardsError::from_error_message(e, "could not unblock board"))
    }
}

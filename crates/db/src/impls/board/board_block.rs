use crate::schema::board_block::dsl::*;
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use crate::{
    models::board::board_block::{BoardBlock, BoardBlockForm},
    traits::Blockable,  
};


impl BoardBlock {
    pub fn read(
        conn: &mut PgConnection,
        for_user_id: i32,
        for_board_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        board_block
            .filter(user_id.eq(for_user_id))
            .filter(board_id.eq(for_board_id))
            .first::<Self>(conn)
            .map_err(|_e| TinyBoardsError::from_string("error reading board block", 500))
    }
}


impl Blockable for BoardBlock {
    type Form = BoardBlockForm;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(board_block)
            .values(form)
            .on_conflict((user_id, board_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|_e| TinyBoardsError::from_string("could not block board", 500))
    }

    fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        diesel::delete(
            board_block
                .filter(user_id.eq(form.user_id))
                .filter(board_id.eq(form.board_id)),
        )
        .execute(conn)
        .map_err(|_e| TinyBoardsError::from_string("could not unblock board", 500))
    }
}
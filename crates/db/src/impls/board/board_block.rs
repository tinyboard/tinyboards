use crate::schema::board_block::dsl::*;
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use crate::{
    models::board::board_block::{BoardBlock, BoardBlockForm},
    traits::Blockable,  
};

impl Blockable for BoardBlock {
    type Form = BoardBlockForm;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(board_block)
            .values(form)
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
use crate::schema::user_board_blocks::dsl::*;
use crate::{
    models::board::user_board_blocks::{BoardBlock, BoardBlockForm},
    traits::Blockable,
    utils::{get_conn, DbPool},
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl BoardBlock {
    pub async fn read(
        pool: &DbPool,
        for_user_id: i32,
        for_board_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        user_board_blocks
            .filter(user_id.eq(for_user_id))
            .filter(board_id.eq(for_board_id))
            .first::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "error reading board block"))
    }
}

#[async_trait::async_trait]
impl Blockable for BoardBlock {
    type Form = BoardBlockForm;
    async fn block(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_board_blocks)
            .values(form)
            .on_conflict((user_id, board_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not block board"))
    }

    async fn unblock(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            user_board_blocks
                .filter(user_id.eq(form.user_id))
                .filter(board_id.eq(form.board_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unblock board"))
    }
}

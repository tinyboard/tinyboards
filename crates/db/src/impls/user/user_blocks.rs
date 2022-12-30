use crate::schema::user_blocks::dsl::*;
use crate::{
    models::user::user_blocks::{UserBlock, UserBlockForm},
    traits::Blockable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;

impl UserBlock {
    pub fn read(
        conn: &mut PgConnection,
        for_user_id: i32,
        for_recipient_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        user_blocks
            .filter(user_id.eq(for_user_id))
            .filter(target_id.eq(for_recipient_id))
            .first::<Self>(conn)
            .map_err(|_| TinyBoardsError::from_message(500, "error reading user block"))
    }
}

impl Blockable for UserBlock {
    type Form = UserBlockForm;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(user_blocks)
            .values(form)
            .on_conflict((user_id, target_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|_| TinyBoardsError::from_message(500, "could not block user"))
    }

    fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        diesel::delete(
            user_blocks
                .filter(user_id.eq(form.user_id))
                .filter(target_id.eq(form.target_id)),
        )
        .execute(conn)
        .map_err(|_| TinyBoardsError::from_message(500, "could not unblock user"))
    }
}

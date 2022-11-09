use crate::schema::user_block::dsl::*;
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use crate::{
    models::user::user_block::{UserBlock, UserBlockForm},
    traits::Blockable,  
};


impl UserBlock {
    pub fn read(
        conn: &mut PgConnection,
        for_user_id: i32,
        for_recipient_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        user_block
            .filter(user_id.eq(for_user_id))
            .filter(target_id.eq(for_recipient_id))
            .first::<Self>(conn)
            .map_err(|_e| TinyBoardsError::from_string("error reading user block", 500))
    }
}


impl Blockable for UserBlock {
    type Form = UserBlockForm;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(user_block)
            .values(form)
            .on_conflict((user_id, target_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|_e| TinyBoardsError::from_string("could not block user", 500))
    }

    fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        diesel::delete(
            user_block
                .filter(user_id.eq(form.user_id))
                .filter(target_id.eq(form.target_id)),
        )
        .execute(conn)
        .map_err(|_e| TinyBoardsError::from_string("could not unblock user", 500))
    }
}
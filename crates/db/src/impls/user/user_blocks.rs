use crate::schema::user_blocks::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::user::user_blocks::{UserBlock, UserBlockForm},
    traits::Blockable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl UserBlock {
    pub async fn read(
        pool: &DbPool,
        for_user_id: i32,
        for_recipient_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        user_blocks
            .filter(user_id.eq(for_user_id))
            .filter(target_id.eq(for_recipient_id))
            .first::<Self>(conn)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "error reading user block"))

    }
}

#[async_trait::async_trait]
impl Blockable for UserBlock {
    type Form = UserBlockForm;
    async fn block(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_blocks)
            .values(form)
            .on_conflict((user_id, target_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not block user"))
    }

    async fn unblock(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            user_blocks
                .filter(user_id.eq(form.user_id))
                .filter(target_id.eq(form.target_id)),
        )
        .execute(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(500, "could not unblock user"))
    }
}

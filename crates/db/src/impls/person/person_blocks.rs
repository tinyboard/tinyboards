use crate::schema::person_blocks::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::person::person_blocks::{PersonBlock, PersonBlockForm},
    traits::Blockable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl PersonBlock {
    pub async fn read(
        pool: &DbPool,
        for_person_id: i32,
        for_recipient_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        person_blocks
            .filter(person_id.eq(for_person_id))
            .filter(target_id.eq(for_recipient_id))
            .first::<Self>(conn)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "error reading user block"))

    }
}

#[async_trait::async_trait]
impl Blockable for PersonBlock {
    type Form = PersonBlockForm;
    async fn block(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(person_blocks)
            .values(form)
            .on_conflict((person_id, target_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "could not block user"))
    }

    async fn unblock(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            person_blocks
                .filter(person_id.eq(form.person_id))
                .filter(target_id.eq(form.target_id)),
        )
        .execute(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(500, "could not unblock user"))
    }
}

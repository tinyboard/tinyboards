use crate::{
    models::board::board_mods::{BoardModerator, BoardModeratorForm},
    traits::Crud, 
    utils::{DbPool, get_conn},
};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

impl BoardModerator {
    pub async fn remove_board_mod(
        pool: &DbPool,
        form: &BoardModeratorForm,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        diesel::delete(
            board_mods
                .filter(board_id.eq(form.board_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .await
    }
}

#[async_trait::async_trait]
impl Crud for BoardModerator {
    type Form = BoardModeratorForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        board_mods.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        diesel::delete(board_mods.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        let new = diesel::insert_into(board_mods)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        diesel::update(board_mods.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

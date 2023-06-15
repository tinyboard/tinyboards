use crate::{
    models::board::board_mods::{BoardModerator, BoardModeratorForm},
    traits::{Crud, Joinable}, 
    utils::{DbPool, get_conn},
};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;
use async_trait::async_trait;

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
                .filter(person_id.eq(form.person_id)),
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

#[async_trait]
impl Joinable for BoardModerator {
  type Form = BoardModeratorForm;
  async fn join(
    pool: &DbPool,
    board_moderator_form: &BoardModeratorForm,
  ) -> Result<Self, Error> {
    use crate::schema::board_mods::dsl::board_mods;
    let conn = &mut get_conn(pool).await?;
    insert_into(board_mods)
      .values(board_moderator_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn leave(
    pool: &DbPool,
    board_moderator_form: &BoardModeratorForm,
  ) -> Result<usize, Error> {
    use crate::schema::board_mods::dsl::{board_id, board_mods, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      board_mods
        .filter(board_id.eq(board_moderator_form.board_id))
        .filter(person_id.eq(board_moderator_form.person_id)),
    )
    .execute(conn)
    .await
  }
}
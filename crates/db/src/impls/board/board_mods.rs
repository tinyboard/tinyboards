use crate::{
    models::board::board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
    traits::{Crud, Joinable},
    utils::{get_conn, DbPool},
};
use async_trait::async_trait;
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;
use std::ops::Add;

impl ModPerms {
    pub fn as_i32(&self) -> i32 {
        use ModPerms::*;

        match self {
            None => 0,
            Config => 2,
            Appearance => 4,
            Content => 8,
            Users => 16,
            Full => 32,
        }
    }
}

impl Add for ModPerms {
    type Output = i32;

    fn add(self, other: Self) -> i32 {
        self.as_i32() + other.as_i32()
    }
}

impl BoardModerator {
    pub async fn remove_board_mod(
        pool: &DbPool,
        person_id_: i32,
        board_id_: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;

        diesel::delete(
            board_mods
                .filter(board_id.eq(board_id_))
                .filter(person_id.eq(person_id_)),
        )
        .execute(conn)
        .await
    }

    pub async fn leave_all_boards(pool: &DbPool, for_person_id: i32) -> Result<usize, Error> {
        use crate::schema::board_mods::dsl::{board_mods, person_id};
        let conn = &mut get_conn(pool).await?;
        diesel::delete(board_mods.filter(person_id.eq(for_person_id)))
            .execute(conn)
            .await
    }

    pub async fn get_person_moderated_boards(
        pool: &DbPool,
        mod_id: i32,
    ) -> Result<Vec<i32>, Error> {
        use crate::schema::board_mods::dsl::{board_id, board_mods, person_id};
        let conn = &mut get_conn(pool).await?;
        board_mods
            .filter(person_id.eq(mod_id))
            .select(board_id)
            .load::<i32>(conn)
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
        board_mods.find(id_).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_mods::dsl::*;
        diesel::delete(board_mods.find(id_)).execute(conn).await
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
    async fn join(pool: &DbPool, board_moderator_form: &BoardModeratorForm) -> Result<Self, Error> {
        use crate::schema::board_mods::dsl::board_mods;
        let conn = &mut get_conn(pool).await?;
        insert_into(board_mods)
            .values(board_moderator_form)
            .get_result::<Self>(conn)
            .await
    }

    async fn leave(pool: &DbPool, form: &BoardModeratorForm) -> Result<usize, Error> {
        use crate::schema::board_mods::dsl::{board_id, board_mods, person_id};

        let person_id_ = form.person_id.unwrap();
        let board_id_ = form.board_id.unwrap();

        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            board_mods
                .filter(board_id.eq(board_id_))
                .filter(person_id.eq(person_id_)),
        )
        .execute(conn)
        .await
    }
}

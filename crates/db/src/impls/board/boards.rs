use crate::newtypes::DbUrl;
use crate::schema::{board_mods, board_person_bans, boards, instance};
use crate::utils::functions::lower;
use crate::{
    models::board::board_person_bans::{BoardPersonBan, BoardPersonBanForm},
    models::board::boards::{Board, BoardForm},
    traits::{Bannable, Crud, ApubActor},
    utils::{get_conn, DbPool, naive_now},
};
use diesel::{dsl::*, prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Board {
    /// Takes a board id and an user id, and returns true if the user mods the board with the given id or is an admin
    pub async fn board_has_mod(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        let mod_id = board_mods::table
            .select(board_mods::id)
            .filter(board_mods::board_id.eq(board_id))
            .filter(board_mods::person_id.eq(person_id))
            .first::<i32>(conn)
            .await
            .optional();

        mod_id.map(|opt| opt.is_some())
    }

    /// Takes a board id and an user id, and returns true if the user is banned from the board with the given id
    pub async fn board_has_ban(
        pool: &DbPool,
        board_id: i32,
        person_id: i32,
    ) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        let ban_id = board_person_bans::table
            .select(board_person_bans::id)
            .filter(board_person_bans::board_id.eq(board_id))
            .filter(board_person_bans::person_id.eq(person_id))
            .filter(
                board_person_bans::expires
                    .is_null()
                    .or(board_person_bans::expires.gt(now)),
            )
            .first::<i32>(conn)
            .await
            .optional();

        ban_id.map(|opt| opt.is_some())
    }

    pub async fn update_deleted(
        pool: &DbPool,
        board_id: i32,
        new_is_deleted: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_deleted.eq(new_is_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }
    
    pub async fn update_banned(
        pool: &DbPool,
        board_id: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::boards::dsl::*;
        diesel::update(boards.find(board_id))
            .set((is_banned.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }
}

pub mod safe_type {
    use crate::{models::board::boards::BoardSafe, schema::boards::*, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        description,
        creation_date,
        updated,
        is_deleted,
        is_nsfw,
        is_hidden,
    );

    impl ToSafe for BoardSafe {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                description,
                creation_date,
                updated,
                is_deleted,
                is_nsfw,
                is_hidden,
            )
        }
    }
}

#[async_trait::async_trait]
impl Crud for Board {
    type Form = BoardForm;
    type IdType = i32;

    async fn read(pool: &DbPool, board_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        boards::table.find(board_id).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, board_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(boards::table.find(board_id)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &BoardForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_board = diesel::insert_into(boards::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_board)
    }
    async fn update(pool: &DbPool, board_id: i32, form: &BoardForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(boards::table.find(board_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Bannable for BoardPersonBan {
    type Form = BoardPersonBanForm;

    async fn ban(pool: &DbPool, ban_form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_person_bans::dsl::{board_id, board_person_bans, person_id};
        insert_into(board_person_bans)
            .values(ban_form)
            .on_conflict((board_id, person_id))
            .do_update()
            .set(ban_form)
            .get_result::<Self>(conn)
            .await
    }

    async fn unban(pool: &DbPool, ban_form: &Self::Form) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_person_bans::dsl::{board_id, board_person_bans, person_id};
        diesel::delete(
            board_person_bans
                .filter(board_id.eq(ban_form.board_id))
                .filter(person_id.eq(ban_form.person_id)),
        )
        .execute(conn)
        .await
    }
}

#[async_trait::async_trait]
impl ApubActor for Board {
    async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        Ok(
          boards::table
            .filter(boards::actor_id.eq(object_id.to_string()))
            .first::<Board>(conn)
            .await
            .ok()
            .map(Into::into),
        )
      }
    
      async fn read_from_name(
        pool: &DbPool,
        board_name: &str,
        include_deleted: bool,
      ) -> Result<Board, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut q = boards::table
          .into_boxed()
          .filter(boards::local.eq(true))
          .filter(lower(boards::name).eq(board_name.to_lowercase()));
        if !include_deleted {
          q = q
            .filter(boards::is_deleted.eq(false));
        }
        q.first::<Self>(conn).await
      }
    
      async fn read_from_name_and_domain(
        pool: &DbPool,
        board_name: &str,
        for_domain: &str,
      ) -> Result<Board, Error> {
        let conn = &mut get_conn(pool).await?;
        boards::table
          .inner_join(instance::table)
          .filter(lower(boards::name).eq(board_name.to_lowercase()))
          .filter(instance::domain.eq(for_domain))
          .select(boards::all_columns)
          .first::<Self>(conn)
          .await
      } 
}
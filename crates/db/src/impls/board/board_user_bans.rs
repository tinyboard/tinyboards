use crate::{
    models::board::board_user_bans::{BoardUserBan, BoardUserBanForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use async_trait::async_trait;
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

impl BoardUserBan {
    /// Check if a user is banned from a specific board
    pub async fn is_banned(pool: &DbPool, user_id_: i32, board_id_: i32) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        let ban_count: i64 = diesel_async::RunQueryDsl::get_result(
            board_user_bans
                .filter(
                    user_id
                        .eq(user_id_)
                        .and(board_id.eq(board_id_))
                        .and(expires.is_null().or(expires.gt(now)))
                )
                .count(),
            conn
        )
        .await?;

        Ok(ban_count > 0)
    }

    /// Get all active bans for a board
    pub async fn for_board(pool: &DbPool, board_id_: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        diesel_async::RunQueryDsl::load(
            board_user_bans
                .filter(
                    board_id
                        .eq(board_id_)
                        .and(expires.is_null().or(expires.gt(now)))
                )
                .order_by(creation_date.desc()),
            conn
        )
        .await
    }

    /// Get all active bans for a user across all boards
    pub async fn for_user(pool: &DbPool, user_id_: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;

        let now = chrono::Utc::now().naive_utc();

        diesel_async::RunQueryDsl::load(
            board_user_bans
                .filter(
                    user_id
                        .eq(user_id_)
                        .and(expires.is_null().or(expires.gt(now)))
                )
                .order_by(creation_date.desc()),
            conn
        )
        .await
    }

    /// Remove a ban (unban a user from a board)
    pub async fn unban_user(pool: &DbPool, user_id_: i32, board_id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;

        diesel_async::RunQueryDsl::execute(
            diesel::delete(
                board_user_bans
                    .filter(user_id.eq(user_id_).and(board_id.eq(board_id_)))
            ),
            conn
        )
        .await
    }

    /// Get a specific ban record
    pub async fn get_ban(pool: &DbPool, user_id_: i32, board_id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;

        diesel_async::RunQueryDsl::first(
            board_user_bans
                .filter(user_id.eq(user_id_).and(board_id.eq(board_id_))),
            conn
        )
        .await
    }
}

#[async_trait]
impl Crud for BoardUserBan {
    type Form = BoardUserBanForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;
        diesel_async::RunQueryDsl::first(board_user_bans.find(id_), conn).await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;
        diesel_async::RunQueryDsl::execute(diesel::delete(board_user_bans.find(id_)), conn).await
    }

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;
        diesel_async::RunQueryDsl::get_result(
            diesel::insert_into(board_user_bans)
                .values(form),
            conn
        )
        .await
    }

    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::board_user_bans::dsl::*;
        diesel_async::RunQueryDsl::get_result(
            diesel::update(board_user_bans.find(id_))
                .set(form),
            conn
        )
        .await
    }
}
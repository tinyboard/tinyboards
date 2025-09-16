use crate::{
    models::user::{UserBoardBlock, UserBoardBlockForm},
    schema::user_board_blocks,
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for UserBoardBlock {
    type Form = UserBoardBlockForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_board_blocks::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn read(pool: &DbPool, block_id: Self::IdType) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_board_blocks::table.find(block_id).first::<Self>(conn).await
    }

    async fn update(pool: &DbPool, block_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_board_blocks::table.find(block_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, block_id: Self::IdType) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_board_blocks::table.find(block_id))
            .execute(conn)
            .await
    }
}

impl UserBoardBlock {
    /// Check if a user has blocked a specific board
    pub async fn is_board_blocked(pool: &DbPool, user_id: i32, board_id: i32) -> Result<bool, Error> {
        use crate::schema::user_board_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let blocked = user_board_blocks
            .filter(user_id.eq(user_id))
            .filter(board_id.eq(board_id))
            .first::<Self>(conn)
            .await;

        Ok(blocked.is_ok())
    }

    /// Get all boards blocked by a specific user
    pub async fn get_blocked_boards(pool: &DbPool, for_user_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_board_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_board_blocks
            .filter(user_id.eq(for_user_id))
            .load::<Self>(conn)
            .await
    }

    /// Remove a board block for a user
    pub async fn unblock_board(pool: &DbPool, user_id: i32, board_id: i32) -> Result<usize, Error> {
        use crate::schema::user_board_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        diesel::delete(
            user_board_blocks
                .filter(user_id.eq(user_id))
                .filter(board_id.eq(board_id))
        )
        .execute(conn)
        .await
    }

    /// Get all users who have blocked a specific board
    pub async fn get_users_blocking_board(pool: &DbPool, for_board_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_board_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_board_blocks
            .filter(board_id.eq(for_board_id))
            .load::<Self>(conn)
            .await
    }
}
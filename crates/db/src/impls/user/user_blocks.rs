use crate::{
    models::user::{UserBlock, UserBlockForm},
    schema::user_blocks,
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for UserBlock {
    type Form = UserBlockForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_blocks::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn read(pool: &DbPool, block_id: Self::IdType) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_blocks::table.find(block_id).first::<Self>(conn).await
    }

    async fn update(pool: &DbPool, block_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_blocks::table.find(block_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, block_id: Self::IdType) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_blocks::table.find(block_id))
            .execute(conn)
            .await
    }
}

impl UserBlock {
    /// Check if a user is blocked by another user
    pub async fn is_blocked(pool: &DbPool, for_user_id: i32, for_target_id: i32) -> Result<bool, Error> {
        use crate::schema::user_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let blocked = user_blocks
            .filter(user_id.eq(for_user_id))
            .filter(target_id.eq(for_target_id))
            .first::<Self>(conn)
            .await;

        Ok(blocked.is_ok())
    }

    /// Get all users blocked by a specific user
    pub async fn get_blocked_users(pool: &DbPool, for_user_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_blocks
            .filter(user_id.eq(for_user_id))
            .load::<Self>(conn)
            .await
    }

    /// Remove a block between two users
    pub async fn unblock(pool: &DbPool, for_user_id: i32, for_target_id: i32) -> Result<usize, Error> {
        use crate::schema::user_blocks::dsl::*;
        let conn = &mut get_conn(pool).await?;

        diesel::delete(
            user_blocks
                .filter(user_id.eq(for_user_id))
                .filter(target_id.eq(for_target_id))
        )
        .execute(conn)
        .await
    }
}
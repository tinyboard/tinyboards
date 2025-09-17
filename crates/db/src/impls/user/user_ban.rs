use crate::{
    models::user::{UserBan, UserBanForm},
    schema::user_ban,
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for UserBan {
    type Form = UserBanForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_ban::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn read(pool: &DbPool, ban_id: Self::IdType) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_ban::table.find(ban_id).first::<Self>(conn).await
    }

    async fn update(pool: &DbPool, ban_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_ban::table.find(ban_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, ban_id: Self::IdType) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_ban::table.find(ban_id))
            .execute(conn)
            .await
    }
}

impl UserBan {
    /// Check if a user is banned
    pub async fn is_banned(pool: &DbPool, user_id: i32) -> Result<bool, Error> {
        use crate::schema::user_ban::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let ban = user_ban
            .filter(user_id.eq(user_id))
            .first::<Self>(conn)
            .await;

        Ok(ban.is_ok())
    }

    /// Get ban record for a specific user
    pub async fn get_user_ban(pool: &DbPool, for_user_id: i32) -> Result<Self, Error> {
        use crate::schema::user_ban::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_ban
            .filter(user_id.eq(for_user_id))
            .first::<Self>(conn)
            .await
    }

    /// Remove a user ban
    pub async fn unban_user(pool: &DbPool, for_user_id: i32) -> Result<usize, Error> {
        use crate::schema::user_ban::dsl::*;
        let conn = &mut get_conn(pool).await?;

        diesel::delete(user_ban.filter(user_id.eq(for_user_id)))
            .execute(conn)
            .await
    }

    /// Get all banned users
    pub async fn get_all_banned_users(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        user_ban::table.load::<Self>(conn).await
    }
}
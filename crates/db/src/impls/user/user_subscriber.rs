use crate::{
    models::user::{UserSubscriber, UserSubscriberForm},
    schema::user_subscriber,
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for UserSubscriber {
    type Form = UserSubscriberForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_subscriber::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn read(pool: &DbPool, subscription_id: Self::IdType) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_subscriber::table.find(subscription_id).first::<Self>(conn).await
    }

    async fn update(pool: &DbPool, subscription_id: Self::IdType, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_subscriber::table.find(subscription_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, subscription_id: Self::IdType) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_subscriber::table.find(subscription_id))
            .execute(conn)
            .await
    }
}

impl UserSubscriber {
    /// Check if a user is following another user
    pub async fn is_following(pool: &DbPool, follower_id: i32, followee_id: i32) -> Result<bool, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let subscription = user_subscriber
            .filter(subscriber_id.eq(follower_id))
            .filter(user_id.eq(followee_id))
            .first::<Self>(conn)
            .await;

        Ok(subscription.is_ok())
    }

    /// Get all users that a specific user is following
    pub async fn get_following(pool: &DbPool, for_user_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_subscriber
            .filter(subscriber_id.eq(for_user_id))
            .load::<Self>(conn)
            .await
    }

    /// Get all followers of a specific user
    pub async fn get_followers(pool: &DbPool, for_user_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_subscriber
            .filter(user_id.eq(for_user_id))
            .load::<Self>(conn)
            .await
    }

    /// Remove a follow relationship between two users
    pub async fn unfollow(pool: &DbPool, follower_id: i32, followee_id: i32) -> Result<usize, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        diesel::delete(
            user_subscriber
                .filter(subscriber_id.eq(follower_id))
                .filter(user_id.eq(followee_id))
        )
        .execute(conn)
        .await
    }

    /// Get pending follow requests for a user
    pub async fn get_pending_requests(pool: &DbPool, for_user_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        user_subscriber
            .filter(user_id.eq(for_user_id))
            .filter(pending.eq(true))
            .load::<Self>(conn)
            .await
    }

    /// Accept a pending follow request
    pub async fn accept_request(pool: &DbPool, follower_id: i32, followee_id: i32) -> Result<Self, Error> {
        use crate::schema::user_subscriber::dsl::*;
        let conn = &mut get_conn(pool).await?;

        diesel::update(
            user_subscriber
                .filter(subscriber_id.eq(follower_id))
                .filter(user_id.eq(followee_id))
        )
        .set(pending.eq(false))
        .get_result::<Self>(conn)
        .await
    }
}
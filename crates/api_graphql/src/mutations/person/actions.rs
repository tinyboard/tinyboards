use crate::{LoggedInUser, PostgresLoader};
use async_graphql::*;
use tinyboards_db::{
    models::person::person_subscriber::{PersonSubscriber, PersonSubscriberForm},
    traits::Subscribeable,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct PersonActions;

#[Object]
impl PersonActions {
    /// Follow a user
    async fn follow_user(
        &self,
        ctx: &Context<'_>,
        person_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if trying to follow yourself
        if user.person.id == person_id {
            return Err(TinyBoardsError::from_message(400, "Cannot follow yourself").into());
        }

        let form = PersonSubscriberForm {
            person_id,
            subscriber_id: user.person.id,
            pending: false,
        };

        PersonSubscriber::subscribe(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to follow user"))?;

        Ok(true)
    }

    /// Unfollow a user
    async fn unfollow_user(
        &self,
        ctx: &Context<'_>,
        person_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let form = PersonSubscriberForm {
            person_id,
            subscriber_id: user.person.id,
            pending: false,
        };

        let rows_affected = PersonSubscriber::unsubscribe(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to unfollow user"))?;

        Ok(rows_affected > 0)
    }

    /// Accept a follow request (for when user profiles are private)
    async fn accept_follow_request(
        &self,
        ctx: &Context<'_>,
        subscriber_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        PersonSubscriber::subscribe_accepted(pool, subscriber_id, user.person.id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to accept follow request"))?;

        Ok(true)
    }
}
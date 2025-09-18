use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{
    models::user::{
        user_subscriber::{UserSubscriber, UserSubscriberForm},
        user_blocks::{UserBlock, UserBlockForm},
        user_board_blocks::{UserBoardBlock, UserBoardBlockForm},
    },
    traits::Crud,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct UserActions;

#[Object]
impl UserActions {
    /// Follow a user
    async fn follow_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if trying to follow yourself
        if user.id == user_id {
            return Err(TinyBoardsError::from_message(400, "Cannot follow yourself").into());
        }

        let form = UserSubscriberForm {
            user_id: Some(user_id),
            subscriber_id: Some(user.id),
            pending: Some(false),
            creation_date: Some(tinyboards_db::utils::naive_now()),
        };

        UserSubscriber::create(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to follow user"))?;

        Ok(true)
    }

    /// Unfollow a user
    async fn unfollow_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let rows_affected = UserSubscriber::unfollow(pool, user.id, user_id).await
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

        UserSubscriber::accept_request(pool, subscriber_id, user.id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to accept follow request"))?;

        Ok(true)
    }

    /// Block a user
    async fn block_user(
        &self,
        ctx: &Context<'_>,
        target_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if trying to block yourself
        if user.id == target_id {
            return Err(TinyBoardsError::from_message(400, "Cannot block yourself").into());
        }

        let form = UserBlockForm {
            user_id: Some(user.id),
            target_id: Some(target_id),
            creation_date: Some(tinyboards_db::utils::naive_now()),
        };

        UserBlock::create(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to block user"))?;

        Ok(true)
    }

    /// Unblock a user
    async fn unblock_user(
        &self,
        ctx: &Context<'_>,
        target_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let rows_affected = UserBlock::unblock(pool, user.id, target_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to unblock user"))?;

        Ok(rows_affected > 0)
    }

    /// Block a board
    async fn block_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let form = UserBoardBlockForm {
            user_id: Some(user.id),
            board_id: Some(board_id),
            creation_date: Some(tinyboards_db::utils::naive_now()),
        };

        UserBoardBlock::create(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to block board"))?;

        Ok(true)
    }

    /// Unblock a board
    async fn unblock_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let rows_affected = UserBoardBlock::unblock_board(pool, user.id, board_id).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to unblock board"))?;

        Ok(rows_affected > 0)
    }
}
use async_graphql::*;
use tinyboards_db::{
    models::stream::{
        stream::Stream as DbStream,
        stream_flair_subscription::{StreamFlairSubscription as DbStreamFlairSubscription, StreamFlairSubscriptionForm},
        stream_board_subscription::{StreamBoardSubscription as DbStreamBoardSubscription, StreamBoardSubscriptionForm},
    },
    utils::DbPool,
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    LoggedInUser,
    structs::stream::{
        AddFlairSubscriptionsInput,
        RemoveFlairSubscriptionInput,
        AddBoardSubscriptionsInput,
        RemoveBoardSubscriptionInput,
        StreamFlairSubscription,
        StreamBoardSubscription,
    },
    helpers::stream::{
        permissions::can_edit_stream,
        feed_generator::{
            validate_subscriptions,
            check_duplicate_flair_subscription,
            check_duplicate_board_subscription,
            get_boards_for_flair_subscriptions,
        },
    },
};

#[derive(Default)]
pub struct StreamSubscriptionMutations;

#[Object]
impl StreamSubscriptionMutations {
    /// Add flair subscriptions to a stream
    async fn add_flair_subscriptions(
        &self,
        ctx: &Context<'_>,
        input: AddFlairSubscriptionsInput,
    ) -> Result<Vec<StreamFlairSubscription>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Validate input
        if input.flair_ids.is_empty() {
            return Err(TinyBoardsError::from_message(400, "At least one flair ID must be provided").into());
        }

        if input.flair_ids.len() > 50 {
            return Err(TinyBoardsError::from_message(400, "Cannot add more than 50 flairs at once").into());
        }

        // Validate flair IDs exist
        validate_subscriptions(pool, &input.flair_ids, &[]).await?;

        // Check subscription quota
        use crate::helpers::stream::permissions::check_subscription_quota;
        check_subscription_quota(&stream, pool, input.flair_ids.len()).await?;

        // Get board IDs for the flairs
        let board_ids = get_boards_for_flair_subscriptions(pool, &input.flair_ids).await?;

        // Add subscriptions, skipping duplicates
        let mut created_subscriptions = Vec::new();

        for (flair_id, board_id) in input.flair_ids.iter().zip(board_ids.iter()) {
            // Skip if already exists
            if check_duplicate_flair_subscription(pool, input.stream_id, *flair_id).await.is_err() {
                continue;
            }

            let form = StreamFlairSubscriptionForm {
                stream_id: input.stream_id,
                flair_id: *flair_id,
                board_id: *board_id,
            };

            if let Ok(subscription) = DbStreamFlairSubscription::create(pool, &form).await {
                created_subscriptions.push(StreamFlairSubscription::from(subscription));
            }
        }

        // Return the created subscriptions (empty vec if all were duplicates)
        // This is not an error - it just means the flairs were already subscribed
        Ok(created_subscriptions)
    }

    /// Remove a flair subscription from a stream
    async fn remove_flair_subscription(
        &self,
        ctx: &Context<'_>,
        input: RemoveFlairSubscriptionInput,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Remove subscription
        DbStreamFlairSubscription::delete(pool, input.stream_id, input.flair_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Flair subscription not found"))?;

        Ok(true)
    }

    /// Add board subscriptions to a stream (subscribe to ALL content from boards)
    async fn add_board_subscriptions(
        &self,
        ctx: &Context<'_>,
        input: AddBoardSubscriptionsInput,
    ) -> Result<Vec<StreamBoardSubscription>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Validate input
        if input.board_ids.is_empty() {
            return Err(TinyBoardsError::from_message(400, "At least one board ID must be provided").into());
        }

        if input.board_ids.len() > 50 {
            return Err(TinyBoardsError::from_message(400, "Cannot add more than 50 boards at once").into());
        }

        // Validate board IDs exist
        validate_subscriptions(pool, &[], &input.board_ids).await?;

        // Check subscription quota
        use crate::helpers::stream::permissions::check_subscription_quota;
        check_subscription_quota(&stream, pool, input.board_ids.len()).await?;

        // Add subscriptions, skipping duplicates
        let mut created_subscriptions = Vec::new();

        for &board_id in &input.board_ids {
            // Skip if already exists
            if check_duplicate_board_subscription(pool, input.stream_id, board_id).await.is_err() {
                continue;
            }

            let form = StreamBoardSubscriptionForm {
                stream_id: input.stream_id,
                board_id,
                include_all_posts: true,  // Default to true for board subscriptions
            };

            if let Ok(subscription) = DbStreamBoardSubscription::create(pool, &form).await {
                created_subscriptions.push(StreamBoardSubscription::from(subscription));
            }
        }

        // Return the created subscriptions (empty vec if all were duplicates)
        // This is not an error - it just means the boards were already subscribed
        Ok(created_subscriptions)
    }

    /// Remove a board subscription from a stream
    async fn remove_board_subscription(
        &self,
        ctx: &Context<'_>,
        input: RemoveBoardSubscriptionInput,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Remove subscription
        DbStreamBoardSubscription::delete(pool, input.stream_id, input.board_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Board subscription not found"))?;

        Ok(true)
    }

    /// Clear all subscriptions from a stream
    async fn clear_stream_subscriptions(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = DbStream::read(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check permissions
        can_edit_stream(Some(user), &stream)?;

        // Delete all flair subscriptions
        DbStreamFlairSubscription::delete_all_for_stream(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to clear flair subscriptions"))?;

        // Delete all board subscriptions
        DbStreamBoardSubscription::delete_all_for_stream(pool, stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to clear board subscriptions"))?;

        Ok(true)
    }
}

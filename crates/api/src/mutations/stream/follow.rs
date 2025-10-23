use async_graphql::*;
use tinyboards_db::{
    models::stream::{
        stream::Stream as DbStream,
        stream_follower::{StreamFollower as DbStreamFollower, StreamFollowerForm},
    },
    utils::DbPool,
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    LoggedInUser,
    structs::stream::{FollowStreamInput, UpdateStreamNavbarInput},
    helpers::stream::permissions::can_view_stream,
};

#[derive(Default)]
pub struct StreamFollowMutations;

#[Object]
impl StreamFollowMutations {
    /// Follow a stream
    async fn follow_stream(
        &self,
        ctx: &Context<'_>,
        input: FollowStreamInput,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Get stream
        let stream = <DbStream as Crud>::read(pool, input.stream_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Stream not found"))?;

        // Check if user can view this stream
        can_view_stream(Some(user), &stream, None, pool).await?;

        // Check if already following
        if DbStreamFollower::is_following(pool, user.id, input.stream_id).await.unwrap_or(false) {
            return Err(TinyBoardsError::from_message(409, "Already following this stream").into());
        }

        // Validate navbar position
        if let Some(pos) = input.navbar_position {
            if pos < 0 || pos > 100 {
                return Err(TinyBoardsError::from_message(400, "navbar_position must be between 0 and 100").into());
            }
        }

        // Create follower relationship
        let form = StreamFollowerForm {
            user_id: user.id,
            stream_id: input.stream_id,
            added_to_navbar: input.add_to_navbar.unwrap_or(false),
            navbar_position: input.navbar_position,
        };

        DbStreamFollower::follow(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to follow stream"))?;

        Ok(true)
    }

    /// Unfollow a stream
    async fn unfollow_stream(
        &self,
        ctx: &Context<'_>,
        stream_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Delete follower relationship
        DbStreamFollower::unfollow(pool, stream_id, user.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Not following this stream"))?;

        Ok(true)
    }

    /// Update navbar settings for a followed stream
    async fn update_stream_navbar_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateStreamNavbarInput,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if following
        if !DbStreamFollower::is_following(pool, user.id, input.stream_id).await.unwrap_or(false) {
            return Err(TinyBoardsError::from_message(404, "Not following this stream").into());
        }

        // Validate navbar position
        if let Some(pos) = input.navbar_position {
            if pos < 0 || pos > 100 {
                return Err(TinyBoardsError::from_message(400, "navbar_position must be between 0 and 100").into());
            }
        }

        // Update navbar settings
        use tinyboards_db::models::stream::stream_follower::UpdateStreamFollowerForm;

        let form = UpdateStreamFollowerForm {
            added_to_navbar: Some(input.add_to_navbar),
            navbar_position: Some(input.navbar_position),
        };

        DbStreamFollower::update_navbar_settings(
            pool,
            input.stream_id,
            user.id,
            &form,
        )
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update navbar settings"))?;

        Ok(true)
    }

    /// Reorder navbar streams (update positions for multiple streams at once)
    async fn reorder_navbar_streams(
        &self,
        ctx: &Context<'_>,
        stream_positions: Vec<NavbarStreamPosition>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Validate positions
        for pos in &stream_positions {
            if pos.position < 0 || pos.position > 100 {
                return Err(TinyBoardsError::from_message(400, "All positions must be between 0 and 100").into());
            }
        }

        // Update each stream's navbar position
        for pos in stream_positions {
            // Skip if not following
            if !DbStreamFollower::is_following(pool, user.id, pos.stream_id).await.unwrap_or(false) {
                continue;
            }

            // Update position
            use tinyboards_db::models::stream::stream_follower::UpdateStreamFollowerForm;

            let form = UpdateStreamFollowerForm {
                added_to_navbar: Some(true), // Keep in navbar
                navbar_position: Some(Some(pos.position)),
            };

            let _ = DbStreamFollower::update_navbar_settings(
                pool,
                pos.stream_id,
                user.id,
                &form,
            )
            .await;
        }

        Ok(true)
    }
}

/// Input for reordering navbar streams
#[derive(InputObject)]
pub struct NavbarStreamPosition {
    pub stream_id: i32,
    pub position: i32,
}

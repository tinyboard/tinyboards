use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::stream::{
        Stream as DbStream, StreamBoardSubscription as DbStreamBoardSubscription,
        StreamBoardSubscriptionInsertForm, StreamFlairSubscription as DbStreamFlairSubscription,
        StreamFlairSubscriptionInsertForm,
    },
    schema::{stream_board_subscriptions, stream_flair_subscriptions, streams},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::stream::{
        AddBoardSubscriptionsInput, AddFlairSubscriptionsInput, StreamBoardSubscription,
        StreamFlairSubscription,
    },
    LoggedInUser,
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
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = input
            .stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;
        let board_uuid: Uuid = input
            .board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Verify ownership
        let stream: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if stream.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the creator can modify subscriptions")
                    .into(),
            );
        }

        let mut created = Vec::new();

        for flair_id in input.flair_ids {
            // Skip duplicates
            let exists: i64 = stream_flair_subscriptions::table
                .filter(stream_flair_subscriptions::stream_id.eq(stream_uuid))
                .filter(stream_flair_subscriptions::flair_id.eq(flair_id))
                .count()
                .get_result(conn)
                .await
                .unwrap_or(0);

            if exists > 0 {
                continue;
            }

            let form = StreamFlairSubscriptionInsertForm {
                stream_id: stream_uuid,
                board_id: board_uuid,
                flair_id,
            };

            if let Ok(sub) = diesel::insert_into(stream_flair_subscriptions::table)
                .values(&form)
                .get_result::<DbStreamFlairSubscription>(conn)
                .await
            {
                created.push(StreamFlairSubscription::from(sub));
            }
        }

        Ok(created)
    }

    /// Remove a flair subscription from a stream
    async fn remove_flair_subscription(
        &self,
        ctx: &Context<'_>,
        stream_id: ID,
        flair_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        // Verify ownership
        let stream: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if stream.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the creator can modify subscriptions")
                    .into(),
            );
        }

        let deleted = diesel::delete(
            stream_flair_subscriptions::table
                .filter(stream_flair_subscriptions::stream_id.eq(stream_uuid))
                .filter(stream_flair_subscriptions::flair_id.eq(flair_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }

    /// Add board subscriptions to a stream
    async fn add_board_subscriptions(
        &self,
        ctx: &Context<'_>,
        input: AddBoardSubscriptionsInput,
    ) -> Result<Vec<StreamBoardSubscription>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = input
            .stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        // Verify ownership
        let stream: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if stream.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the creator can modify subscriptions")
                    .into(),
            );
        }

        let mut created = Vec::new();

        for board_id in input.board_ids {
            let board_uuid: Uuid = match board_id.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };

            // Skip duplicates
            let exists: i64 = stream_board_subscriptions::table
                .filter(stream_board_subscriptions::stream_id.eq(stream_uuid))
                .filter(stream_board_subscriptions::board_id.eq(board_uuid))
                .count()
                .get_result(conn)
                .await
                .unwrap_or(0);

            if exists > 0 {
                continue;
            }

            let form = StreamBoardSubscriptionInsertForm {
                stream_id: stream_uuid,
                board_id: board_uuid,
                include_all_posts: true,
            };

            if let Ok(sub) = diesel::insert_into(stream_board_subscriptions::table)
                .values(&form)
                .get_result::<DbStreamBoardSubscription>(conn)
                .await
            {
                created.push(StreamBoardSubscription::from(sub));
            }
        }

        Ok(created)
    }

    /// Remove a board subscription from a stream
    async fn remove_board_subscription(
        &self,
        ctx: &Context<'_>,
        stream_id: ID,
        board_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;
        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Verify ownership
        let stream: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if stream.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the creator can modify subscriptions")
                    .into(),
            );
        }

        let deleted = diesel::delete(
            stream_board_subscriptions::table
                .filter(stream_board_subscriptions::stream_id.eq(stream_uuid))
                .filter(stream_board_subscriptions::board_id.eq(board_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(deleted > 0)
    }

    /// Clear all subscriptions from a stream
    async fn clear_stream_subscriptions(
        &self,
        ctx: &Context<'_>,
        stream_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let stream: DbStream = streams::table
            .find(stream_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        if stream.creator_id != user.id && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "Only the creator can modify subscriptions")
                    .into(),
            );
        }

        diesel::delete(
            stream_flair_subscriptions::table
                .filter(stream_flair_subscriptions::stream_id.eq(stream_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        diesel::delete(
            stream_board_subscriptions::table
                .filter(stream_board_subscriptions::stream_id.eq(stream_uuid)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}

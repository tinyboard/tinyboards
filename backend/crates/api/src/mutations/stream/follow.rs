use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::stream::{StreamFollowerInsertForm, StreamFollowerUpdateForm},
    schema::{stream_followers, streams},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct StreamFollowMutations;

#[Object]
impl StreamFollowMutations {
    /// Follow a stream
    async fn follow_stream(&self, ctx: &Context<'_>, stream_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        // Verify stream exists
        let _stream = streams::table
            .find(stream_uuid)
            .first::<tinyboards_db::models::stream::Stream>(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Stream not found".into()))?;

        // Check if already following
        let existing: i64 = stream_followers::table
            .filter(stream_followers::stream_id.eq(stream_uuid))
            .filter(stream_followers::user_id.eq(user.id))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing > 0 {
            return Err(
                TinyBoardsError::from_message(409, "Already following this stream").into(),
            );
        }

        let form = StreamFollowerInsertForm {
            stream_id: stream_uuid,
            user_id: user.id,
            added_to_navbar: false,
            navbar_position: None,
        };

        diesel::insert_into(stream_followers::table)
            .values(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Unfollow a stream
    async fn unfollow_stream(&self, ctx: &Context<'_>, stream_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let deleted = diesel::delete(
            stream_followers::table
                .filter(stream_followers::stream_id.eq(stream_uuid))
                .filter(stream_followers::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if deleted == 0 {
            return Err(
                TinyBoardsError::from_message(404, "Not following this stream").into(),
            );
        }

        Ok(true)
    }

    /// Toggle navbar pin and update position for a followed stream
    async fn update_stream_navbar_settings(
        &self,
        ctx: &Context<'_>,
        stream_id: ID,
        added_to_navbar: bool,
        navbar_position: Option<i32>,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let stream_uuid: Uuid = stream_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid stream ID".into()))?;

        let form = StreamFollowerUpdateForm {
            added_to_navbar: Some(added_to_navbar),
            navbar_position: Some(navbar_position),
        };

        let updated = diesel::update(
            stream_followers::table
                .filter(stream_followers::stream_id.eq(stream_uuid))
                .filter(stream_followers::user_id.eq(user.id)),
        )
        .set(&form)
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if updated == 0 {
            return Err(
                TinyBoardsError::from_message(404, "Not following this stream").into(),
            );
        }

        Ok(true)
    }
}

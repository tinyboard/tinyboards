use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    schema::board_subscribers,
    utils::{DbPool, get_conn},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::helpers::permissions;

#[derive(Default)]
pub struct BoardActions;

#[Object]
impl BoardActions {
    /// Subscribe to a board
    async fn subscribe_to_board(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_auth_not_banned(ctx)?;
        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(board_subscribers::table)
            .values((
                board_subscribers::board_id.eq(board_uuid),
                board_subscribers::user_id.eq(user.id),
                board_subscribers::is_pending.eq(false),
            ))
            .on_conflict((board_subscribers::board_id, board_subscribers::user_id))
            .do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    /// Unsubscribe from a board
    async fn unsubscribe_from_board(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = permissions::require_auth_not_banned(ctx)?;
        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let conn = &mut get_conn(pool).await?;

        let rows_affected = diesel::delete(
            board_subscribers::table
                .filter(board_subscribers::board_id.eq(board_uuid))
                .filter(board_subscribers::user_id.eq(user.id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(rows_affected > 0)
    }
}

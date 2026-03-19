use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::social::{
        UserFollowInsertForm, UserBlockInsertForm, BoardBlockInsertForm,
    },
    schema::{user_follows, user_blocks, board_blocks},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::helpers::permissions;

#[derive(Default)]
pub struct UserActions;

#[Object]
impl UserActions {
    pub async fn follow_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let target_id: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        if target_id == me.id {
            return Err(TinyBoardsError::BadRequest("Cannot follow yourself".to_string()).into());
        }

        let form = UserFollowInsertForm {
            user_id: target_id,
            follower_id: me.id,
            is_pending: false,
        };

        diesel::insert_into(user_follows::table)
            .values(&form)
            .on_conflict((user_follows::user_id, user_follows::follower_id))
            .do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    pub async fn unfollow_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let target_id: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        diesel::delete(
            user_follows::table
                .filter(user_follows::user_id.eq(target_id))
                .filter(user_follows::follower_id.eq(me.id))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    pub async fn block_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let target_id: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        if target_id == me.id {
            return Err(TinyBoardsError::BadRequest("Cannot block yourself".to_string()).into());
        }

        let form = UserBlockInsertForm {
            user_id: me.id,
            target_id,
        };

        diesel::insert_into(user_blocks::table)
            .values(&form)
            .on_conflict((user_blocks::user_id, user_blocks::target_id))
            .do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Also unfollow the blocked user
        diesel::delete(
            user_follows::table
                .filter(user_follows::user_id.eq(target_id))
                .filter(user_follows::follower_id.eq(me.id))
        )
        .execute(conn)
        .await
        .ok();

        Ok(true)
    }

    pub async fn unblock_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let target_id: Uuid = user_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        diesel::delete(
            user_blocks::table
                .filter(user_blocks::user_id.eq(me.id))
                .filter(user_blocks::target_id.eq(target_id))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    pub async fn block_board(&self, ctx: &Context<'_>, board_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let bid: Uuid = board_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        let form = BoardBlockInsertForm {
            user_id: me.id,
            board_id: bid,
        };

        diesel::insert_into(board_blocks::table)
            .values(&form)
            .on_conflict((board_blocks::user_id, board_blocks::board_id))
            .do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }

    pub async fn unblock_board(&self, ctx: &Context<'_>, board_id: ID) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let bid: Uuid = board_id.parse().map_err(|_| TinyBoardsError::BadRequest("Invalid UUID".to_string()))?;

        diesel::delete(
            board_blocks::table
                .filter(board_blocks::user_id.eq(me.id))
                .filter(board_blocks::board_id.eq(bid))
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}

use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{
    models::board::board_subscriber::BoardSubscriberForm,
    traits::Subscribeable,
    models::board::board_subscriber::BoardSubscriber,
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct BoardActions;

#[Object]
impl BoardActions {
    /// Subscribe to a board
    async fn subscribe_to_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let form = BoardSubscriberForm {
            board_id,
            user_id: user.id,
            pending: Some(false),
        };

        BoardSubscriber::subscribe(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to subscribe to board"))?;

        Ok(true)
    }

    /// Unsubscribe from a board
    async fn unsubscribe_from_board(
        &self,
        ctx: &Context<'_>,
        board_id: i32,
    ) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        let form = BoardSubscriberForm {
            board_id,
            user_id: user.id,
            pending: Some(false),
        };

        let rows_affected = BoardSubscriber::unsubscribe(pool, &form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to unsubscribe from board"))?;

        Ok(rows_affected > 0)
    }
}
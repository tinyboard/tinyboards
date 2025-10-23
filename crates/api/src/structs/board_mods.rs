use crate::PostgresLoader;
use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::models::board::board_mods::BoardModerator as DbBoardMod;
use tinyboards_utils::TinyBoardsError;

use crate::newtypes::{BoardId, UserId};

use super::{boards::Board, user::User};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BoardMod {
    id: i32,
    board_id: i32,
    user_id: i32,
    created_at: String,
    permissions: i32,
    rank: i32,
    invite_accepted: bool,
    invite_accepted_date: Option<String>,
}

#[ComplexObject]
impl BoardMod {
    pub async fn user(&self, ctx: &Context<'_>) -> Result<User> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader.load_one(UserId(self.user_id)).await.map(|opt| {
            opt.ok_or_else(|| {
                TinyBoardsError::from_message(
                    500,
                    "Failed to load user for board mod relationship.",
                )
                .into()
            })
        })?
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Board> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader.load_one(BoardId(self.board_id)).await.map(|opt| {
            opt.ok_or_else(|| {
                TinyBoardsError::from_message(
                    500,
                    "Failed to load board for board mod relationship.",
                )
                .into()
            })
        })?
    }
}

impl From<DbBoardMod> for BoardMod {
    fn from(value: DbBoardMod) -> Self {
        Self {
            id: value.id,
            board_id: value.board_id,
            user_id: value.user_id,
            created_at: value.creation_date.to_string(),
            permissions: value.permissions,
            rank: value.rank,
            invite_accepted: value.invite_accepted,
            invite_accepted_date: value.invite_accepted_date.map(|t| t.to_string()),
        }
    }
}

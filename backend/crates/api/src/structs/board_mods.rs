use crate::PostgresLoader;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;
use tinyboards_db::models::board::board_mods::BoardModerator as DbBoardMod;
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::newtypes::{BoardId, UserId};

use super::{boards::Board, user::User};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BoardMod {
    id: ID,
    board_id: ID,
    user_id: ID,
    creation_date: String,
    permissions: i32,
    rank: i32,
    invite_accepted: bool,
    invite_accepted_date: Option<String>,
    #[graphql(skip)]
    uuid_board_id: Uuid,
    #[graphql(skip)]
    uuid_user_id: Uuid,
}

#[ComplexObject]
impl BoardMod {
    pub async fn user(&self, ctx: &Context<'_>) -> Result<User> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader.load_one(UserId(self.uuid_user_id)).await.map(|opt| {
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

        loader.load_one(BoardId(self.uuid_board_id)).await.map(|opt| {
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
            id: ID(value.id.to_string()),
            board_id: ID(value.board_id.to_string()),
            user_id: ID(value.user_id.to_string()),
            creation_date: value.created_at.to_rfc3339(),
            permissions: value.permissions,
            rank: value.rank,
            invite_accepted: value.is_invite_accepted,
            invite_accepted_date: value.invite_accepted_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
            uuid_board_id: value.board_id,
            uuid_user_id: value.user_id,
        }
    }
}

use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::models::board::board_mods::BoardModerator as DbBoardMod;
use tinyboards_utils::TinyBoardsError;

use crate::{
    newtypes::{BoardId, PersonId},
    PostgresLoader,
};

use super::{boards::Board, person::Person};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BoardMod {
    id: i32,
    board_id: i32,
    person_id: i32,
    creation_date: String,
    permissions: i32,
    rank: i32,
    invite_accepted: bool,
    invite_accepted_date: Option<String>,
}

#[ComplexObject]
impl BoardMod {
    pub async fn person(&self, ctx: &Context<'_>) -> Result<Person> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader.load_one(PersonId(self.person_id)).await.map(|opt| {
            opt.ok_or_else(|| {
                TinyBoardsError::from_message(
                    500,
                    "Failed to load person for board mod relationship.",
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
            person_id: value.person_id,
            creation_date: value.creation_date.to_string(),
            permissions: value.permissions,
            rank: value.rank,
            invite_accepted: value.invite_accepted,
            invite_accepted_date: value.invite_accepted_date.map(|t| t.to_string()),
        }
    }
}

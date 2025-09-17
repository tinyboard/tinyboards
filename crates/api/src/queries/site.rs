use async_graphql::*;
use tinyboards_db::{models::site::site::Site as DbSite, utils::DbPool};
use tinyboards_utils::TinyBoardsError;

use crate::structs::site::LocalSite;

#[derive(Default)]
pub struct QuerySite;

#[Object]
impl QuerySite {
    pub async fn site(&self, ctx: &Context<'_>) -> Result<LocalSite> {
        let pool = ctx.data_unchecked::<DbPool>();

        DbSite::read(pool)
            .await
            .map(LocalSite::from)
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load site, sorry :(").into()
            })
    }
}

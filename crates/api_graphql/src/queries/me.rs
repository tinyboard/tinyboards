use crate::{structs::person::Person, LoggedInUser};
use async_graphql::*;
use tinyboards_db::utils::DbPool;
//use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct MeQuery;

#[Object]
impl MeQuery {
    pub async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Person> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;

        Ok(Person::from(v.clone()))
    }

    pub async fn unread_replies_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        v.local_user
            .as_ref()
            .unwrap()
            .get_unread_replies_count(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn unread_mentions_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        v.local_user
            .as_ref()
            .unwrap()
            .get_unread_mentions_count(pool)
            .await
            .map_err(|e| e.into())
    }
}

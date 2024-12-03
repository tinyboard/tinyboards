use crate::{structs::person::Person, LoggedInUser};
use async_graphql::*;
use tinyboards_db::utils::DbPool;
use tinyboards_db_views::structs::{CommentReplyView, PersonMentionView};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct MeQuery;

#[Object]
impl MeQuery {
    pub async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Person> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;

        Ok(Person::from((&v.person, &v.counts)))
    }

    pub async fn unread_replies_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        CommentReplyView::get_unread_replies(pool, v.person.id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Loading unread replies count failed.")
                    .into()
            })
    }

    pub async fn unread_mentions_count<'ctx>(&self, ctx: &Context<'ctx>) -> Result<i64> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        PersonMentionView::get_unread_mentions(pool, v.person.id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Loading unread mention count failed.")
                    .into()
            })
    }
}

use crate::{
    structs::person::{Me, Person},
    LoggedInUser,
};
use async_graphql::*;
use tinyboards_db::{models::board::boards::Board as DbBoard, utils::DbPool};
use tinyboards_db_views::structs::{CommentReplyView, PersonMentionView};
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct MeQuery;

#[Object]
impl MeQuery {
    pub async fn me<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Me> {
        let v = ctx.data::<LoggedInUser>()?.require_user()?;
        let pool = ctx.data::<DbPool>()?;

        let my_person = if ctx.look_ahead().field("person").exists() {
            Some(Person::from((&v.person, &v.counts)))
        } else {
            None
        };

        let unread_replies_count = if ctx.look_ahead().field("unreadRepliesCount").exists() {
            let count = CommentReplyView::get_unread_replies(pool, v.person.id)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Loading unread replies count failed.",
                    )
                })?;

            Some(count)
        } else {
            None
        };

        let unread_mentions_count = if ctx.look_ahead().field("unreadMentionsCount").exists() {
            let count = PersonMentionView::get_unread_mentions(pool, v.person.id)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Loading unread mention count failed.",
                    )
                })?;

            Some(count)
        } else {
            None
        };

        Ok(Me {
            person: my_person,
            unread_replies_count,
            unread_mentions_count,
        })
    }
}

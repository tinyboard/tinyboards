use async_graphql::*;
use tinyboards_db::{
    models::{
        person::notifications::Notification as DbNotification,
        person::person::Person,
        post::posts::Post,
        comment::comments::Comment,
    },
    traits::Crud,
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::{
    structs::{comment::Comment as GqlComment, person::Person as GqlPerson, post::Post as GqlPost},
    LoggedInUser,
};

#[derive(Default)]
pub struct QueryNotifications;

#[derive(SimpleObject)]
pub struct Notification {
    pub id: i32,
    pub kind: String,
    pub is_read: bool,
    pub created: String,
    pub comment: Option<GqlComment>,
    pub post: Option<GqlPost>,
    pub person: Option<GqlPerson>,
}

#[Object]
impl QueryNotifications {
    /// Get user notifications
    pub async fn get_notifications(
        &self,
        ctx: &Context<'_>,
        unread_only: Option<bool>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Notification>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let unread_only = unread_only.unwrap_or(false);
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(25).min(50); // Cap at 50
        let offset = (page - 1) * limit;

        use tinyboards_db::schema::notifications;
        
        let mut query = notifications::table
            .filter(notifications::recipient_id.eq(user.person.id))
            .into_boxed();

        if unread_only {
            query = query.filter(notifications::is_read.eq(false));
        }

        let db_notifications = query
            .order(notifications::created.desc())
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<DbNotification>(conn)
            .await?;

        let mut result = Vec::new();
        for notification in db_notifications {
            // Load related comment/post/person if needed
            let comment = if let Some(comment_id) = notification.comment_id {
                match Comment::get_with_counts(pool, comment_id).await {
                    Ok(comment) => Some(GqlComment::from(comment)),
                    Err(_) => None,
                }
            } else {
                None
            };

            let post = if let Some(post_id) = notification.post_id {
                match Post::get_with_counts(pool, post_id, false).await {
                    Ok(post) => Some(GqlPost::from(post)),
                    Err(_) => None,
                }
            } else {
                None
            };

            // For now, we'll leave person as None since we don't have a direct person reference in notifications
            let person = None;

            result.push(Notification {
                id: notification.id,
                kind: notification.kind,
                is_read: notification.is_read,
                created: notification.created.to_string(),
                comment,
                post,
                person,
            });
        }

        Ok(result)
    }
}
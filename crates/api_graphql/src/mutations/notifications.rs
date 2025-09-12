use async_graphql::*;
use tinyboards_db::{
    models::person::notifications::{Notification as DbNotification, NotificationForm},
    traits::Crud,
    utils::{DbPool, get_conn},
};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct NotificationMutations;

#[derive(SimpleObject)]
pub struct MarkNotificationsReadResponse {
    pub success: bool,
    pub marked_count: i32,
}

#[Object]
impl NotificationMutations {
    /// Mark notifications as read
    pub async fn mark_notifications_read(
        &self,
        ctx: &Context<'_>,
        notification_ids: Option<Vec<i32>>,
        mark_all: Option<bool>,
    ) -> Result<MarkNotificationsReadResponse> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let mark_all = mark_all.unwrap_or(false);

        use tinyboards_db::schema::notifications;

        let marked_count = if mark_all {
            // Mark all notifications as read for this user
            diesel::update(notifications::table)
                .filter(notifications::recipient_id.eq(user.person.id))
                .filter(notifications::is_read.eq(false))
                .set(notifications::is_read.eq(true))
                .execute(conn)
                .await? as i32
        } else if let Some(ids) = notification_ids {
            if ids.is_empty() {
                return Ok(MarkNotificationsReadResponse {
                    success: true,
                    marked_count: 0,
                });
            }

            // Mark specific notifications as read (only ones owned by user)
            diesel::update(notifications::table)
                .filter(notifications::id.eq_any(ids))
                .filter(notifications::recipient_id.eq(user.person.id))
                .filter(notifications::is_read.eq(false))
                .set(notifications::is_read.eq(true))
                .execute(conn)
                .await? as i32
        } else {
            return Err(TinyBoardsError::from_message(
                400,
                "Either notification_ids or mark_all must be provided",
            )
            .into());
        };

        Ok(MarkNotificationsReadResponse {
            success: true,
            marked_count,
        })
    }
}
use crate::{
    models::notification::notifications::{Notification, NotificationForm},
    schema::notifications,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl Notification {
    pub async fn create(pool: &DbPool, form: &NotificationForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::insert_into(notifications::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read(pool: &DbPool, notification_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        notifications::table
            .find(notification_id)
            .first::<Self>(conn)
            .await
    }

    pub async fn mark_as_read(pool: &DbPool, notification_id: i32, user_id: i32) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(notifications::table)
            .filter(notifications::id.eq(notification_id))
            .filter(notifications::recipient_user_id.eq(user_id))
            .set(notifications::is_read.eq(true))
            .get_result::<Self>(conn)
            .await
            .map_err(|e| match e {
                Error::NotFound => TinyBoardsError::from_message(404, "Notification not found"),
                _ => TinyBoardsError::from_error_message(e, 500, "Failed to mark notification as read"),
            })
    }

    pub async fn mark_many_as_read(
        pool: &DbPool,
        notification_ids: Vec<i32>,
        user_id: i32
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(notifications::table)
            .filter(notifications::id.eq_any(notification_ids))
            .filter(notifications::recipient_user_id.eq(user_id))
            .filter(notifications::is_read.eq(false))
            .set(notifications::is_read.eq(true))
            .execute(conn)
            .await
    }

    pub async fn mark_all_as_read(pool: &DbPool, user_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(notifications::table)
            .filter(notifications::recipient_user_id.eq(user_id))
            .filter(notifications::is_read.eq(false))
            .set(notifications::is_read.eq(true))
            .execute(conn)
            .await
    }

    pub async fn delete(pool: &DbPool, notification_id: i32, user_id: i32) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::delete(notifications::table)
            .filter(notifications::id.eq(notification_id))
            .filter(notifications::recipient_user_id.eq(user_id))
            .execute(conn)
            .await
            .map_err(|e| match e {
                Error::NotFound => TinyBoardsError::from_message(404, "Notification not found"),
                _ => TinyBoardsError::from_error_message(e, 500, "Failed to delete notification"),
            })
    }

    pub async fn get_for_user(
        pool: &DbPool,
        user_id: i32,
        unread_only: Option<bool>,
        kind_filter: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;

        let mut query = notifications::table
            .filter(notifications::recipient_user_id.eq(user_id))
            .into_boxed();

        if let Some(true) = unread_only {
            query = query.filter(notifications::is_read.eq(false));
        }

        if let Some(kind) = kind_filter {
            query = query.filter(notifications::kind.eq(kind));
        }

        query = query.order(notifications::created.desc());

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query.load::<Self>(conn).await
    }

    pub async fn count_unread_for_user(pool: &DbPool, user_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;

        notifications::table
            .filter(notifications::recipient_user_id.eq(user_id))
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
    }

    pub async fn count_unread_by_kind(
        pool: &DbPool,
        user_id: i32
    ) -> Result<Vec<(String, i64)>, Error> {
        let conn = &mut get_conn(pool).await?;

        notifications::table
            .filter(notifications::recipient_user_id.eq(user_id))
            .filter(notifications::is_read.eq(false))
            .group_by(notifications::kind)
            .select((notifications::kind, diesel::dsl::count(notifications::id)))
            .load::<(String, i64)>(conn)
            .await
    }

    pub async fn delete_old_notifications(
        pool: &DbPool,
        days_old: i32,
    ) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        let cutoff_date = chrono::Utc::now().naive_utc() - chrono::Duration::days(days_old as i64);

        diesel::delete(notifications::table)
            .filter(notifications::created.lt(cutoff_date))
            .execute(conn)
            .await
    }
}
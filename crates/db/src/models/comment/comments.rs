use crate::schema::comments;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub parent_id: Option<i32>,
    pub body: String,
    pub body_html: String,
    pub is_removed: bool,
    pub read: bool,
    pub creation_date: NaiveDateTime,
    pub level: i32,
    pub is_deleted: bool,
    pub updated: Option<NaiveDateTime>,
    pub is_locked: bool,
    pub board_id: i32,
    pub language_id: Option<i32>,
    pub is_pinned: Option<bool>,
    pub approval_status: String,
    pub approved_by: Option<i32>,
    pub approved_at: Option<NaiveDateTime>,
    pub creator_vote: i32,
    pub quoted_comment_id: Option<i32>,
    pub slug: String,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comments)]
pub struct CommentForm {
    pub creator_id: Option<i32>,
    pub post_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub is_removed: Option<bool>,
    pub read: Option<bool>,
    pub level: Option<i32>,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: Option<bool>,
    pub board_id: Option<i32>,
    pub language_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub is_pinned: Option<Option<bool>>,
    pub approval_status: Option<String>,
    pub approved_by: Option<i32>,
    pub approved_at: Option<NaiveDateTime>,
    pub slug: Option<String>,
    pub creator_vote: Option<i32>,
    pub quoted_comment_id: Option<i32>,
}

impl Comment {
    /// Update comment approval status
    pub async fn update_approval_status(
        pool: &crate::utils::DbPool,
        comment_id: i32,
        status: &str,
        approved_by: Option<i32>,
    ) -> Result<Self, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        let now = chrono::Utc::now().naive_utc();

        use diesel_async::RunQueryDsl;

        let updated_comment = diesel_async::RunQueryDsl::get_result(
            diesel::update(comments::table.find(comment_id))
                .set((
                    comments::approval_status.eq(status),
                    comments::approved_by.eq(approved_by),
                    comments::approved_at.eq(if status == "approved" { Some(now) } else { None }),
                )),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(updated_comment)
    }

    /// Update comment removed status for moderation
    pub async fn update_removed_status(
        pool: &crate::utils::DbPool,
        comment_id: i32,
        is_removed: bool,
    ) -> Result<Self, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        use diesel_async::RunQueryDsl;

        let updated_comment = diesel_async::RunQueryDsl::get_result(
            diesel::update(comments::table.find(comment_id))
                .set(comments::is_removed.eq(is_removed)),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(updated_comment)
    }

    /// Get comments pending approval
    pub async fn get_pending_approval(
        pool: &crate::utils::DbPool,
        board_id: Option<i32>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Self>, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        let mut query = comments::table.into_boxed();

        query = query.filter(comments::approval_status.eq("pending"));

        if let Some(board_id) = board_id {
            query = query.filter(comments::board_id.eq(board_id));
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        use diesel_async::RunQueryDsl;

        let comments = diesel_async::RunQueryDsl::load(
            query
                .order(comments::creation_date.desc())
                .limit(limit as i64)
                .offset(offset as i64),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(comments)
    }
}

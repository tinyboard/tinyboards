use crate::{schema::{comment_report, comments}, utils::{get_conn, DbPool}};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tinyboards_utils::TinyBoardsError;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = comment_report)]
pub struct CommentReport {
    pub id: i32,
    pub creator_id: i32,
    pub comment_id: i32,
    pub original_comment_text: String,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = comment_report)]
pub struct CommentReportForm {
    pub creator_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub original_comment_text: Option<String>,
    pub reason: Option<String>,
    pub resolved: Option<bool>,
    pub resolver_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
}

impl CommentReport {
    pub async fn read(pool: &DbPool, report_id: i32) -> Result<Self, TinyBoardsError> {
        use crate::schema::comment_report::dsl::*;
        let conn = &mut get_conn(pool).await?;

        comment_report
            .filter(id.eq(report_id))
            .first::<Self>(conn)
            .await
            .map_err(TinyBoardsError::from)
    }

    pub async fn list(
        pool: &DbPool,
        board_id_filter: Option<i32>,
        resolved_only: Option<bool>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        use crate::schema::comment_report::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let mut query = comment_report.into_boxed();

        // If filtering by board_id, we need to get comment IDs for that board first
        if let Some(board_id) = board_id_filter {
            let comment_ids: Vec<i32> = comments::table
                .filter(comments::board_id.eq(board_id))
                .select(comments::id)
                .load::<i32>(conn)
                .await
                .map_err(TinyBoardsError::from)?;

            query = query.filter(comment_id.eq_any(comment_ids));
        }

        if let Some(resolved_filter) = resolved_only {
            query = query.filter(resolved.eq(resolved_filter));
        }

        query = query.order(creation_date.desc());

        if let Some(limit) = limit {
            query = query.limit(limit.into());
        }

        if let Some(offset) = offset {
            query = query.offset(offset.into());
        }

        query
            .load::<Self>(conn)
            .await
            .map_err(TinyBoardsError::from)
    }
}

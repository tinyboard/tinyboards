use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use crate::{schema::{post_report, posts}, newtypes::DbUrl, utils::{get_conn, DbPool}};
use chrono::NaiveDateTime;
use tinyboards_utils::TinyBoardsError;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[derive(Queryable, Identifiable)]
#[diesel(table_name = post_report)]
pub struct PostReport {
    pub id: i32,
    pub creator_id: i32,
    pub post_id: i32,
    pub original_post_title: String,
    pub original_post_url: Option<DbUrl>,
    pub original_post_body: Option<String>,
    pub reason: String,
    pub resolved: bool,
    pub resolver_id: Option<i32>,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Clone, Insertable, AsChangeset, Default)]
#[diesel(table_name = post_report)]
pub struct PostReportForm {
    pub creator_id: Option<i32>,
    pub post_id: Option<i32>,
    pub original_post_title: Option<String>,
    pub original_post_url: Option<DbUrl>,
    pub original_post_body: Option<String>,
    pub reason: Option<String>,
    pub resolved: Option<bool>,
    pub resolver_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
}

impl PostReport {
    pub async fn read(pool: &DbPool, report_id: i32) -> Result<Self, TinyBoardsError> {
        use crate::schema::post_report::dsl::*;
        let conn = &mut get_conn(pool).await?;

        post_report
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
        use crate::schema::post_report::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let mut query = post_report.into_boxed();

        // If filtering by board_id, we need to get post IDs for that board first
        if let Some(board_id) = board_id_filter {
            let post_ids: Vec<i32> = posts::table
                .filter(posts::board_id.eq(board_id))
                .select(posts::id)
                .load::<i32>(conn)
                .await
                .map_err(TinyBoardsError::from)?;

            query = query.filter(post_id.eq_any(post_ids));
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
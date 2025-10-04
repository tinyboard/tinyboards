use crate::{newtypes::DbUrl, schema::posts};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub type_: String,
    pub url: Option<DbUrl>,
    pub thumbnail_url: Option<DbUrl>,
    pub permalink: Option<DbUrl>,
    pub body: String,
    pub body_html: String,
    pub creator_id: i32,
    pub board_id: i32,
    pub is_removed: bool,
    pub is_locked: bool,
    pub creation_date: NaiveDateTime,
    pub is_deleted: bool,
    pub is_nsfw: bool,
    pub updated: Option<NaiveDateTime>,
    pub image: Option<DbUrl>,
    pub language_id: Option<i32>,
    pub featured_board: bool,
    pub featured_local: bool,
    pub alt_text: Option<String>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_video_url: Option<DbUrl>,
    pub source_url: Option<DbUrl>,
    pub last_crawl_date: Option<NaiveDateTime>,
    pub title_chunk: String,
    pub approval_status: String,
    pub approved_by: Option<i32>,
    pub approved_at: Option<NaiveDateTime>,
    pub creator_vote: i32,
    pub post_type: String,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = posts)]
pub struct PostForm {
    pub title: Option<String>,
    pub type_: Option<String>,
    pub url: Option<DbUrl>,
    pub thumbnail_url: Option<DbUrl>,
    pub permalink: Option<Option<DbUrl>>,
    pub body: Option<String>,
    pub body_html: Option<String>,
    pub creator_id: Option<i32>,
    pub board_id: Option<i32>,
    pub is_removed: Option<bool>,
    pub is_locked: Option<bool>,
    pub updated: Option<NaiveDateTime>,
    pub is_deleted: Option<bool>,
    pub is_nsfw: Option<bool>,
    pub image: Option<DbUrl>,
    pub language_id: Option<i32>,
    pub creation_date: Option<NaiveDateTime>,
    pub featured_board: Option<bool>,
    pub featured_local: Option<bool>,
    pub alt_text: Option<String>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_video_url: Option<DbUrl>,
    pub source_url: Option<DbUrl>,
    pub last_crawl_date: Option<NaiveDateTime>,
    pub title_chunk: Option<String>,
    pub approval_status: Option<String>,
    pub approved_by: Option<i32>,
    pub approved_at: Option<NaiveDateTime>,
    pub creator_vote: Option<i32>,
    pub post_type: Option<String>,
}

impl Post {
    /// Update post approval status
    pub async fn update_approval_status(
        pool: &crate::utils::DbPool,
        post_id: i32,
        status: &str,
        approved_by: Option<i32>,
    ) -> Result<Self, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        let now = chrono::Utc::now().naive_utc();

        use diesel_async::RunQueryDsl;

        let updated_post = diesel_async::RunQueryDsl::get_result(
            diesel::update(posts::table.find(post_id))
                .set((
                    posts::approval_status.eq(status),
                    posts::approved_by.eq(approved_by),
                    posts::approved_at.eq(if status == "approved" { Some(now) } else { None }),
                )),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(updated_post)
    }

    /// Update post removed status for moderation
    pub async fn update_removed_status(
        pool: &crate::utils::DbPool,
        post_id: i32,
        is_removed: bool,
    ) -> Result<Self, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        use diesel_async::RunQueryDsl;

        let updated_post = diesel_async::RunQueryDsl::get_result(
            diesel::update(posts::table.find(post_id))
                .set(posts::is_removed.eq(is_removed)),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(updated_post)
    }

    /// Get posts pending approval
    pub async fn get_pending_approval(
        pool: &crate::utils::DbPool,
        board_id: Option<i32>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Self>, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        let conn = &mut get_conn(pool).await?;

        let mut query = posts::table.into_boxed();

        query = query.filter(posts::approval_status.eq("pending"));

        if let Some(board_id) = board_id {
            query = query.filter(posts::board_id.eq(board_id));
        }

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        use diesel_async::RunQueryDsl;

        let posts = diesel_async::RunQueryDsl::load(
            query
                .order(posts::creation_date.desc())
                .limit(limit as i64)
                .offset(offset as i64),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(posts)
    }
}

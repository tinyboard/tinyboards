use crate::models::site::content_uploads::{ContentUpload, ContentUploadForm, ContentUploadView};
use crate::models::site::uploads::Upload;
use crate::schema::{content_uploads, uploads};
use crate::traits::Crud;
use crate::utils::DbPool;
use crate::utils::get_conn;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for ContentUpload {
    type Form = ContentUploadForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        content_uploads::table.find(id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(content_uploads::table.find(id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(content_uploads::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(content_uploads::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

impl ContentUpload {
    /// Get all content uploads for a specific post
    pub async fn get_by_post(pool: &DbPool, post_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        content_uploads::table
            .filter(content_uploads::post_id.eq(post_id))
            .order(content_uploads::position.asc())
            .load::<Self>(conn)
            .await
    }

    /// Get all content uploads for a specific comment
    pub async fn get_by_comment(pool: &DbPool, comment_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        content_uploads::table
            .filter(content_uploads::comment_id.eq(comment_id))
            .order(content_uploads::position.asc())
            .load::<Self>(conn)
            .await
    }

    /// Get all content uploads for a post with full upload details
    pub async fn get_by_post_with_uploads(
        pool: &DbPool,
        post_id: i32,
    ) -> Result<Vec<ContentUploadView>, Error> {
        let conn = &mut get_conn(pool).await?;
        content_uploads::table
            .inner_join(uploads::table.on(content_uploads::upload_id.eq(uploads::id)))
            .filter(content_uploads::post_id.eq(post_id))
            .order(content_uploads::position.asc())
            .select((
                content_uploads::id,
                content_uploads::upload_id,
                content_uploads::post_id,
                content_uploads::comment_id,
                content_uploads::created_at,
                content_uploads::position,
                uploads::file_name,
                uploads::original_name,
                uploads::upload_url,
                uploads::size,
            ))
            .load::<(i32, i32, Option<i32>, Option<i32>, chrono::NaiveDateTime, Option<i32>, String, String, crate::newtypes::DbUrl, i64)>(conn)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, upload_id, post_id, comment_id, created_at, position, file_name, original_name, upload_url, size)| {
                        ContentUploadView {
                            id,
                            upload_id,
                            post_id,
                            comment_id,
                            created_at,
                            position,
                            file_name,
                            original_name,
                            upload_url,
                            size,
                        }
                    })
                    .collect()
            })
    }

    /// Get all content uploads for a comment with full upload details
    pub async fn get_by_comment_with_uploads(
        pool: &DbPool,
        comment_id: i32,
    ) -> Result<Vec<ContentUploadView>, Error> {
        let conn = &mut get_conn(pool).await?;
        content_uploads::table
            .inner_join(uploads::table.on(content_uploads::upload_id.eq(uploads::id)))
            .filter(content_uploads::comment_id.eq(comment_id))
            .order(content_uploads::position.asc())
            .select((
                content_uploads::id,
                content_uploads::upload_id,
                content_uploads::post_id,
                content_uploads::comment_id,
                content_uploads::created_at,
                content_uploads::position,
                uploads::file_name,
                uploads::original_name,
                uploads::upload_url,
                uploads::size,
            ))
            .load::<(i32, i32, Option<i32>, Option<i32>, chrono::NaiveDateTime, Option<i32>, String, String, crate::newtypes::DbUrl, i64)>(conn)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, upload_id, post_id, comment_id, created_at, position, file_name, original_name, upload_url, size)| {
                        ContentUploadView {
                            id,
                            upload_id,
                            post_id,
                            comment_id,
                            created_at,
                            position,
                            file_name,
                            original_name,
                            upload_url,
                            size,
                        }
                    })
                    .collect()
            })
    }

    /// Delete all content uploads for a post
    pub async fn delete_by_post(pool: &DbPool, post_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(content_uploads::table.filter(content_uploads::post_id.eq(post_id)))
            .execute(conn)
            .await
    }

    /// Delete all content uploads for a comment
    pub async fn delete_by_comment(pool: &DbPool, comment_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(content_uploads::table.filter(content_uploads::comment_id.eq(comment_id)))
            .execute(conn)
            .await
    }

    /// Delete a specific upload association
    pub async fn delete_by_upload(pool: &DbPool, upload_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(content_uploads::table.filter(content_uploads::upload_id.eq(upload_id)))
            .execute(conn)
            .await
    }
}

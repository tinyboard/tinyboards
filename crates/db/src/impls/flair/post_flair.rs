use crate::{
    models::flair::post_flair::{PostFlair, PostFlairForm},
    schema::post_flairs,
    traits::Crud,
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl PostFlair {
    /// Get flair for a specific post
    pub async fn get_by_post(pool: &DbPool, post_id: i32) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        post_flairs::table
            .filter(post_flairs::post_id.eq(post_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get flairs for multiple posts
    pub async fn get_for_posts(
        pool: &DbPool,
        post_ids: Vec<i32>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        post_flairs::table
            .filter(post_flairs::post_id.eq_any(post_ids))
            .load::<Self>(conn)
            .await
    }

    /// Assign or update flair for a post
    pub async fn assign_to_post(
        pool: &DbPool,
        post_id: i32,
        form: &PostFlairForm,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        // Check if flair already exists for this post
        let existing = Self::get_by_post(pool, post_id).await?;

        if let Some(existing) = existing {
            // Update existing flair
            diesel::update(post_flairs::table.find(existing.id))
                .set(form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update post flair"))
        } else {
            // Create new flair
            let mut new_form = form.clone();
            new_form.post_id = Some(post_id);

            diesel::insert_into(post_flairs::table)
                .values(&new_form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to assign post flair"))
        }
    }

    /// Remove flair from a post
    pub async fn remove_from_post(pool: &DbPool, post_id: i32) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(post_flairs::table.filter(post_flairs::post_id.eq(post_id)))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove post flair"))
    }

    /// Get posts with a specific flair template
    pub async fn get_posts_with_template(
        pool: &DbPool,
        template_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = post_flairs::table
            .filter(post_flairs::flair_template_id.eq(template_id))
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query
            .order_by(post_flairs::creation_date.desc())
            .load::<Self>(conn)
            .await
    }

    /// Count posts using a specific flair template
    pub async fn count_with_template(pool: &DbPool, template_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        post_flairs::table
            .filter(post_flairs::flair_template_id.eq(template_id))
            .count()
            .get_result::<i64>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for PostFlair {
    type Form = PostFlairForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        post_flairs::table.find(id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(post_flairs::table.find(id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &PostFlairForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(post_flairs::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id: i32, form: &PostFlairForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(post_flairs::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

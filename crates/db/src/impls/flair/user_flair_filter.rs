use crate::{
    models::flair::user_flair_filter::{UserFlairFilter, UserFlairFilterForm},
    schema::user_flair_filters,
    traits::Crud,
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl UserFlairFilter {
    /// Get filter for a user in a specific board
    pub async fn get_by_user_and_board(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flair_filters::table
            .filter(user_flair_filters::user_id.eq(user_id))
            .filter(user_flair_filters::board_id.eq(board_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get all filters for a user
    pub async fn get_for_user(pool: &DbPool, user_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flair_filters::table
            .filter(user_flair_filters::user_id.eq(user_id))
            .load::<Self>(conn)
            .await
    }

    /// Create or update a filter for a user in a board
    pub async fn set_filter(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
        form: &UserFlairFilterForm,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        // Check if filter already exists
        let existing = Self::get_by_user_and_board(pool, user_id, board_id).await?;

        let mut new_form = form.clone();
        new_form.user_id = Some(user_id);
        new_form.board_id = Some(board_id);

        if let Some(existing) = existing {
            // Update existing filter
            diesel::update(user_flair_filters::table.find(existing.id))
                .set(&new_form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "Failed to update flair filter")
                })
        } else {
            // Create new filter
            diesel::insert_into(user_flair_filters::table)
                .values(&new_form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "Failed to create flair filter")
                })
        }
    }

    /// Remove filter for a user in a board
    pub async fn remove_filter(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
    ) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            user_flair_filters::table
                .filter(user_flair_filters::user_id.eq(user_id))
                .filter(user_flair_filters::board_id.eq(board_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove flair filter"))
    }

    /// Add flair IDs to included list
    pub async fn add_to_included(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
        flair_ids: Vec<i32>,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let filter = Self::get_by_user_and_board(pool, user_id, board_id)
            .await?
            .ok_or_else(|| {
                TinyBoardsError::from_message(404, "Flair filter not found for this board")
            })?;

        let mut current_ids = filter.included_flair_ids.clone();
        for id in flair_ids {
            if !current_ids.contains(&Some(id)) {
                current_ids.push(Some(id));
            }
        }

        diesel::update(user_flair_filters::table.find(filter.id))
            .set((
                user_flair_filters::included_flair_ids.eq(current_ids),
                user_flair_filters::updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to update included flairs")
            })
    }

    /// Add flair IDs to excluded list
    pub async fn add_to_excluded(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
        flair_ids: Vec<i32>,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        let filter = Self::get_by_user_and_board(pool, user_id, board_id)
            .await?
            .ok_or_else(|| {
                TinyBoardsError::from_message(404, "Flair filter not found for this board")
            })?;

        let mut current_ids = filter.excluded_flair_ids.clone();
        for id in flair_ids {
            if !current_ids.contains(&Some(id)) {
                current_ids.push(Some(id));
            }
        }

        diesel::update(user_flair_filters::table.find(filter.id))
            .set((
                user_flair_filters::excluded_flair_ids.eq(current_ids),
                user_flair_filters::updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to update excluded flairs")
            })
    }

    /// Check if a post with given flair should be shown to user
    pub async fn should_show_post(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
        flair_template_id: Option<i32>,
    ) -> Result<bool, TinyBoardsError> {
        // If no flair on post, always show
        let Some(flair_id) = flair_template_id else {
            return Ok(true);
        };

        let filter = Self::get_by_user_and_board(pool, user_id, board_id).await?;

        // If no filter set, show everything
        let Some(filter) = filter else {
            return Ok(true);
        };

        match filter.filter_mode.as_str() {
            "hide" => {
                // Hide mode: hide posts with flairs in excluded list
                if !filter.excluded_flair_ids.is_empty() {
                    Ok(!filter.excluded_flair_ids.contains(&Some(flair_id)))
                } else {
                    Ok(true)
                }
            }
            "show" => {
                // Show mode: only show posts with flairs in included list
                if !filter.included_flair_ids.is_empty() {
                    Ok(filter.included_flair_ids.contains(&Some(flair_id)))
                } else {
                    Ok(false)
                }
            }
            _ => Ok(true),
        }
    }
}

#[async_trait::async_trait]
impl Crud for UserFlairFilter {
    type Form = UserFlairFilterForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flair_filters::table.find(id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_flair_filters::table.find(id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &UserFlairFilterForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_flair_filters::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id: i32, form: &UserFlairFilterForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_flair_filters::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

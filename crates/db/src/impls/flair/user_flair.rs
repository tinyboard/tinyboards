use crate::{
    models::flair::user_flair::{UserFlair, UserFlairForm},
    schema::user_flairs,
    traits::Crud,
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl UserFlair {
    /// Get user flair for a specific user in a board
    pub async fn get_by_user_and_board(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
    ) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flairs::table
            .filter(user_flairs::user_id.eq(user_id))
            .filter(user_flairs::board_id.eq(board_id))
            .first::<Self>(conn)
            .await
            .optional()
    }

    /// Get flairs for multiple users in a board
    pub async fn get_for_users_in_board(
        pool: &DbPool,
        user_ids: Vec<i32>,
        board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flairs::table
            .filter(user_flairs::user_id.eq_any(user_ids))
            .filter(user_flairs::board_id.eq(board_id))
            .filter(user_flairs::is_approved.eq(true))
            .load::<Self>(conn)
            .await
    }

    /// Assign or update user flair (requires approval if template requires it)
    pub async fn assign_to_user(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
        form: &UserFlairForm,
        auto_approve: bool,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        // Check if flair already exists
        let existing = Self::get_by_user_and_board(pool, user_id, board_id).await?;

        let mut new_form = form.clone();
        new_form.user_id = Some(user_id);
        new_form.board_id = Some(board_id);

        if auto_approve {
            new_form.is_approved = Some(true);
        } else {
            new_form.is_approved = Some(false);
        }

        if let Some(existing) = existing {
            // Update existing flair
            diesel::update(user_flairs::table.find(existing.id))
                .set(&new_form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to update user flair"))
        } else {
            // Create new flair
            diesel::insert_into(user_flairs::table)
                .values(&new_form)
                .get_result::<Self>(conn)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to assign user flair"))
        }
    }

    /// Remove user flair
    pub async fn remove_from_user(
        pool: &DbPool,
        user_id: i32,
        board_id: i32,
    ) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            user_flairs::table
                .filter(user_flairs::user_id.eq(user_id))
                .filter(user_flairs::board_id.eq(board_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove user flair"))
    }

    /// Approve user flair
    pub async fn approve(
        pool: &DbPool,
        flair_id: i32,
        approver_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_flairs::table.find(flair_id))
            .set((
                user_flairs::is_approved.eq(true),
                user_flairs::approved_by.eq(Some(approver_id)),
                user_flairs::approved_at.eq(Some(naive_now())),
            ))
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to approve user flair"))
    }

    /// Reject user flair
    pub async fn reject(
        pool: &DbPool,
        flair_id: i32,
        rejector_id: i32,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_flairs::table.find(flair_id))
            .set((
                user_flairs::is_approved.eq(false),
                user_flairs::approved_by.eq(Some(rejector_id)),
            ))
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to reject user flair"))
    }

    /// Get pending user flairs for a board (not approved)
    pub async fn get_pending_for_board(
        pool: &DbPool,
        board_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = user_flairs::table
            .filter(user_flairs::board_id.eq(board_id))
            .filter(user_flairs::is_approved.eq(false))
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query
            .order_by(user_flairs::assigned_at.asc())
            .load::<Self>(conn)
            .await
    }

    /// Get users with a specific flair template
    pub async fn get_users_with_template(
        pool: &DbPool,
        template_id: i32,
        board_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = user_flairs::table
            .filter(user_flairs::flair_template_id.eq(template_id))
            .filter(user_flairs::board_id.eq(board_id))
            .filter(user_flairs::is_approved.eq(true))
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query
            .order_by(user_flairs::assigned_at.desc())
            .load::<Self>(conn)
            .await
    }

    /// Count users with a specific flair template
    pub async fn count_with_template(
        pool: &DbPool,
        template_id: i32,
        board_id: i32,
    ) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flairs::table
            .filter(user_flairs::flair_template_id.eq(template_id))
            .filter(user_flairs::board_id.eq(board_id))
            .filter(user_flairs::is_approved.eq(true))
            .count()
            .get_result::<i64>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for UserFlair {
    type Form = UserFlairForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        user_flairs::table.find(id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(user_flairs::table.find(id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &UserFlairForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_flairs::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id: i32, form: &UserFlairForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(user_flairs::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

use crate::{
    models::flair::{
        flair_template::{FlairTemplate, FlairTemplateForm},
        flair_aggregates::FlairAggregates,
    },
    schema::flair_templates,
    traits::Crud,
    utils::{get_conn, naive_now, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl FlairTemplate {
    /// Get all flair templates for a board filtered by type
    pub async fn get_by_board(
        pool: &DbPool,
        board_id: i32,
        flair_type: Option<&str>,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = flair_templates::table
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::is_active.eq(true))
            .into_boxed();

        if let Some(flair_type) = flair_type {
            query = query.filter(flair_templates::flair_type.eq(flair_type));
        }

        query
            .order_by(flair_templates::display_order.asc())
            .load::<Self>(conn)
            .await
    }

    /// Get user-assignable flair templates for a board
    pub async fn get_user_assignable(
        pool: &DbPool,
        board_id: i32,
        flair_type: &str,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_templates::table
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::flair_type.eq(flair_type))
            .filter(flair_templates::mod_only.eq(false))
            .filter(flair_templates::is_active.eq(true))
            .order_by(flair_templates::display_order.asc())
            .load::<Self>(conn)
            .await
    }

    /// Get mod-only flair templates for a board
    pub async fn get_mod_only(
        pool: &DbPool,
        board_id: i32,
        flair_type: &str,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_templates::table
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::flair_type.eq(flair_type))
            .filter(flair_templates::mod_only.eq(true))
            .filter(flair_templates::is_active.eq(true))
            .order_by(flair_templates::display_order.asc())
            .load::<Self>(conn)
            .await
    }

    /// Soft delete a flair template
    pub async fn soft_delete(pool: &DbPool, template_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(flair_templates::table.find(template_id))
            .set((
                flair_templates::is_active.eq(false),
                flair_templates::updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Reorder flair templates
    pub async fn reorder(
        pool: &DbPool,
        template_id: i32,
        new_order: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(flair_templates::table.find(template_id))
            .set((
                flair_templates::display_order.eq(new_order),
                flair_templates::updated_at.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    /// Get flair template by ID if not deleted
    pub async fn get_active(pool: &DbPool, template_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_templates::table
            .find(template_id)
            .filter(flair_templates::is_active.eq(true))
            .first::<Self>(conn)
            .await
    }

    /// Check if user can assign a flair template
    pub async fn can_user_assign(
        pool: &DbPool,
        template_id: i32,
        is_mod: bool,
    ) -> Result<bool, TinyBoardsError> {
        let template = Self::get_active(pool, template_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Flair template not found"))?;

        if template.mod_only && !is_mod {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get templates with their usage counts
    pub async fn get_with_counts(
        pool: &DbPool,
        board_id: i32,
        flair_type: Option<&str>,
    ) -> Result<Vec<(Self, crate::models::flair::FlairAggregates)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::flair_aggregates;

        // Load templates first
        let mut query = flair_templates::table
            .filter(flair_templates::board_id.eq(board_id))
            .filter(flair_templates::is_active.eq(true))
            .into_boxed();

        if let Some(flair_type) = flair_type {
            query = query.filter(flair_templates::flair_type.eq(flair_type));
        }

        let templates = query
            .order_by(flair_templates::display_order.asc())
            .load::<Self>(conn)
            .await?;

        // Load aggregates separately for each template
        let mut results = Vec::new();
        for template in templates {
            let aggregate = flair_aggregates::table
                .filter(flair_aggregates::flair_template_id.eq(template.id))
                .first::<FlairAggregates>(conn)
                .await?;
            results.push((template, aggregate));
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl Crud for FlairTemplate {
    type Form = FlairTemplateForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        flair_templates::table.find(id).first::<Self>(conn).await
    }

    async fn delete(pool: &DbPool, id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(flair_templates::table.find(id))
            .execute(conn)
            .await
    }

    async fn create(pool: &DbPool, form: &FlairTemplateForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(flair_templates::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id: i32, form: &FlairTemplateForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(flair_templates::table.find(id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

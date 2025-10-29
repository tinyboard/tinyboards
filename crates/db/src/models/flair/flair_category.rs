use crate::schema::flair_categories;
use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = flair_categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FlairCategory {
    pub id: i32,
    pub board_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = flair_categories)]
pub struct CreateFlairCategory {
    pub board_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: Option<i32>,
    pub created_by: i32,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = flair_categories)]
pub struct UpdateFlairCategory {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub display_order: Option<i32>,
}

impl FlairCategory {
    pub async fn create(
        pool: &crate::utils::DbPool,
        form: CreateFlairCategory,
    ) -> Result<Self, Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        diesel::insert_into(flair_categories)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read(pool: &crate::utils::DbPool, category_id: i32) -> Result<Self, Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        flair_categories.find(category_id).first::<Self>(conn).await
    }

    pub async fn update(
        pool: &crate::utils::DbPool,
        category_id: i32,
        form: UpdateFlairCategory,
    ) -> Result<Self, Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        diesel::update(flair_categories.find(category_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn delete(pool: &crate::utils::DbPool, category_id: i32) -> Result<usize, Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        diesel::delete(flair_categories.find(category_id))
            .execute(conn)
            .await
    }

    pub async fn for_board(
        pool: &crate::utils::DbPool,
        board_id_param: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        flair_categories
            .filter(board_id.eq(board_id_param))
            .order(display_order.asc())
            .load::<Self>(conn)
            .await
    }

    pub async fn reorder(
        pool: &crate::utils::DbPool,
        updates: Vec<(i32, i32)>, // Vec<(category_id, new_order)>
    ) -> Result<(), Error> {
        use crate::schema::flair_categories::dsl::*;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        for (category_id, new_order) in updates {
            diesel::update(flair_categories.find(category_id))
                .set(display_order.eq(new_order))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    /// Get count of flairs in this category
    pub async fn get_flair_count(pool: &crate::utils::DbPool, cat_id: i32) -> Result<i64, Error> {
        use crate::schema::flair_templates;
        let conn = &mut pool.get().await.map_err(|_| Error::NotFound)?;

        flair_templates::table
            .filter(flair_templates::category_id.eq(cat_id))
            .filter(flair_templates::is_active.eq(true))
            .count()
            .get_result(conn)
            .await
    }
}

use crate::schema::post_hidden;
use crate::traits::Crud;
use crate::utils::{get_conn, DbPool};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{dsl::exists, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tinyboards_utils::TinyBoardsError;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = post_hidden)]
pub struct PostHidden {
    pub id: i32,
    pub post_id: i32,
    pub user_id: i32,
    pub creation_date: NaiveDateTime,
}

#[derive(Clone, Default, Insertable, AsChangeset)]
#[diesel(table_name = post_hidden)]
pub struct PostHiddenForm {
    pub post_id: i32,
    pub user_id: i32,
}

#[async_trait::async_trait]
impl Crud for PostHidden {
    type Form = PostHiddenForm;
    type IdType = i32;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, diesel::result::Error> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(post_hidden)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_nothing()
            .get_result::<Self>(conn)
            .await
    }

    async fn read(pool: &DbPool, hidden_id: Self::IdType) -> Result<Self, diesel::result::Error> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        post_hidden.find(hidden_id).first::<Self>(conn).await
    }

    async fn update(
        pool: &DbPool,
        hidden_id: Self::IdType,
        form: &Self::Form,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(post_hidden.find(hidden_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, hidden_id: Self::IdType) -> Result<usize, diesel::result::Error> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(post_hidden.find(hidden_id)).execute(conn).await
    }
}

impl PostHidden {
    pub async fn hide(pool: &DbPool, form: &PostHiddenForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, form)
            .await
            .map_err(TinyBoardsError::from)
    }

    pub async fn unhide(pool: &DbPool, form: &PostHiddenForm) -> Result<usize, TinyBoardsError> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            post_hidden
                .filter(post_id.eq(form.post_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .await
        .map_err(TinyBoardsError::from)
    }

    pub async fn is_hidden_by_user(
        pool: &DbPool,
        post_id_param: i32,
        user_id_param: i32,
    ) -> Result<bool, TinyBoardsError> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;
        select(exists(
            post_hidden
                .filter(post_id.eq(post_id_param))
                .filter(user_id.eq(user_id_param)),
        ))
        .get_result(conn)
        .await
        .map_err(TinyBoardsError::from)
    }

    pub async fn get_hidden_posts_for_user(
        pool: &DbPool,
        user_id_param: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<i32>, TinyBoardsError> {
        use crate::schema::post_hidden::dsl::*;
        let conn = &mut get_conn(pool).await?;

        let mut query = post_hidden
            .filter(user_id.eq(user_id_param))
            .order(creation_date.desc())
            .select(post_id)
            .into_boxed();

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        query.load::<i32>(conn).await.map_err(TinyBoardsError::from)
    }
}
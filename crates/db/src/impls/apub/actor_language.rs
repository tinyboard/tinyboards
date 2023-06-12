use crate::schema::{local_user_language, site_language, board_language};
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::actor_language::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

pub const UNDETERMINED_ID: i32 = 0;

#[async_trait::async_trait]
impl Crud for LocalUserLanguage {
    type Form = LocalUserLanguageForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        local_user_language::table.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(local_user_language::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id_: i32, form: &Self::Form,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user_language::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_user_language::table.find(id_)).execute(conn)
        .await
    }
}

#[async_trait::async_trait]
impl Crud for SiteLanguage {
    type Form = SiteLanguageForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        site_language::table.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(site_language::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id_: i32, form: &Self::Form,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(site_language::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(site_language::table.find(id_)).execute(conn)
        .await
    }
}













#[async_trait::async_trait]
impl Crud for BoardLanguage {
    type Form = BoardLanguageForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        board_language::table.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(board_language::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(pool: &DbPool, id_: i32, form: &Self::Form,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(board_language::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(board_language::table.find(id_)).execute(conn)
        .await
    }
}
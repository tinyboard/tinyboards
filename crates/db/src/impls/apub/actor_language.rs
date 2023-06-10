use crate::schema::local_user_language::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::person::local_user_language::*,
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
        local_user_language.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &LocalUserLanguageForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(local_user_language)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &LocalUserLanguageForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user_language.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_user_language.find(id_)).execute(conn)
        .await
    }
}
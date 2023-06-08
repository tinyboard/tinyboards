use crate::schema::local_site_rate_limit::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::local_site_rate_limit::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for LocalSiteRateLimit {
    type Form = LocalSiteRateLimitForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        local_site_rate_limit.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &LocalSiteRateLimitForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(local_site_rate_limit)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &LocalSiteRateLimitForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_site_rate_limit.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_site_rate_limit.find(id_)).execute(conn)
        .await
    }
}
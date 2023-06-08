use crate::schema::local_site::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::local_site::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for LocalSite {
    type Form = LocalSiteForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        local_site.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &LocalSiteForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(local_site)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &LocalSiteForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_site.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_site.find(id_)).execute(conn)
        .await
    }
}

impl LocalSite {
    pub async fn read_local(pool: &DbPool) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_site::dsl::*;
        local_site.order_by(id).first::<Self>(conn)
        .await
    }
}
use crate::schema::federation_allowlist::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::federation_allowlist::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for FederationAllowList {
    type Form = FederationAllowListForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        federation_allowlist.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &FederationAllowListForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(federation_allowlist)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &FederationAllowListForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(federation_allowlist.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(federation_allowlist.find(id_)).execute(conn)
        .await
    }
}
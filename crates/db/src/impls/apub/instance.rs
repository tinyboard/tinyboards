use crate::schema::{instance, federation_allowlist, federation_blocklist};
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::instance::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for Instance {
    type Form = InstanceForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        instance::table.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &InstanceForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(instance::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &InstanceForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(instance::table.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(instance::table.find(id_)).execute(conn)
        .await
    }
}

impl Instance {

    pub async fn allow_list(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        instance::table
            .inner_join(federation_allowlist::table)
            .select(instance::all_columns)
            .get_results(conn)
            .await
    }

    pub async fn block_list(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        instance::table
            .inner_join(federation_blocklist::table)
            .select(instance::all_columns)
            .get_results(conn)
            .await        
    }

}
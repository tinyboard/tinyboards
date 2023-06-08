use crate::schema::instance::dsl::*;
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
        instance.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &InstanceForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(instance)
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
        diesel::update(instance.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(instance.find(id_)).execute(conn)
        .await
    }
}
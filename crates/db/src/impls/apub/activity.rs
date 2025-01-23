use crate::newtypes::DbUrl;
use crate::schema::activity::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::apub::activity::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for Activity {
    type Form = ActivityForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        activity.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &ActivityForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(activity)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &ActivityForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(activity.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(activity.find(id_)).execute(conn)
        .await
    }
}

impl Activity {
    pub async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Activity, Error> {
      let conn = &mut get_conn(pool).await?;
      activity
        .filter(ap_id.eq(object_id))
        .first::<Self>(conn)
        .await
    }
  }
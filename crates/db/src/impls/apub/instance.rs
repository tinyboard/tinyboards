use crate::schema::{instance, federation_allowlist, federation_blocklist};
use crate::utils::{get_conn, DbPool, naive_now};
use crate::{
    models::apub::instance::*,
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::{RunQueryDsl, AsyncPgConnection};

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

    pub(crate) async fn read_or_create_with_conn(
        conn: &mut AsyncPgConnection,
        domain_: String,
      ) -> Result<Self, Error> {
        // First try to read the instance row and return directly if found
        let instance = instance::table
          .filter(instance::domain.eq(&domain_))
          .first::<Self>(conn)
          .await;
        match instance {
          Ok(i) => Ok(i),
          Err(diesel::NotFound) => {
            // Instance not in database yet, insert it
            let form = InstanceForm {
                domain: Some(domain_),
                updated: Some(naive_now()),
            };

            insert_into(instance::table)
              .values(&form)
              // Necessary because this method may be called concurrently for the same domain. This
              // could be handled with a transaction, but nested transactions arent allowed
              .on_conflict(instance::domain)
              .do_update()
              .set(&form)
              .get_result::<Self>(conn)
              .await
          }
          e => e,
        }
      }

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

    pub async fn read_or_create(pool: &DbPool, domain: String) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        Self::read_or_create_with_conn(conn, domain).await
    }

    pub async fn linked(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        instance::table
          .left_join(federation_blocklist::table)
          .filter(federation_blocklist::id.is_null())
          .select(instance::all_columns)
          .get_results(conn)
          .await
      }

}
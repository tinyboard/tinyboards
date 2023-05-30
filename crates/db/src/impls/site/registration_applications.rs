use crate::schema::registration_applications::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::site::registration_applications::{
        RegistrationApplication, RegistrationApplicationForm,
    },
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for RegistrationApplication {
    type Form = RegistrationApplicationForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        registration_applications.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &RegistrationApplicationForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(registration_applications)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &RegistrationApplicationForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(registration_applications.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(registration_applications.find(id_)).execute(conn)
        .await
    }
}

impl RegistrationApplication {
    pub async fn find_by_person_id(pool: &DbPool, person_id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        registration_applications
            .filter(person_id.eq(person_id_))
            .first::<Self>(conn)
            .await
    }
}

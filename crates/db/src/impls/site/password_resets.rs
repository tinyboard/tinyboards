use crate::schema::password_resets::dsl::*;
use crate::utils::{DbPool, get_conn};
use crate::{
    models::site::password_resets::{
        PasswordReset, PasswordResetForm,
    },
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;

impl PasswordReset {
    pub async fn get_by_token(pool: &DbPool, token: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::password_resets::dsl::*;
        password_resets
            .filter(reset_token.eq(token))
            .first::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for PasswordReset {
    type Form = PasswordResetForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        password_resets.find(id_).first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, form: &PasswordResetForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        insert_into(password_resets)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn update(
        pool: &DbPool,
        id_: i32,
        form: &PasswordResetForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(password_resets.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(password_resets.find(id_)).execute(conn)
        .await
    }
}
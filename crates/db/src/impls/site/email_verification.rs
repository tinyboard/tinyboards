
use crate::{models::site::email_verification::*, traits::Crud, utils::{get_conn, DbPool}};
use diesel::{result::Error, *};
use crate::schema::email_verification::dsl::*;
use diesel_async::RunQueryDsl;


impl EmailVerification {
    pub async fn read_for_token(pool: &DbPool, token: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        email_verification
            .filter(verification_code.eq(token))
            .first::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for EmailVerification {
    type Form = EmailVerificationForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::email_verification::dsl::*;
        email_verification.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::email_verification::dsl::*;
        diesel::delete(email_verification.find(id_))
        .execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::email_verification::dsl::*;
        let new = diesel::insert_into(email_verification)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::email_verification::dsl::*;
        diesel::update(email_verification.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

use crate::{models::site::email_verification::*, traits::Crud};
use diesel::{result::Error, *};


impl Crud for EmailVerification {
    type Form = EmailVerificationForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::email_verification::dsl::*;
        email_verification.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::email_verification::dsl::*;
        diesel::delete(email_verification.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::email_verification::dsl::*;
        let new = diesel::insert_into(email_verification)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::email_verification::dsl::*;
        diesel::update(email_verification.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}
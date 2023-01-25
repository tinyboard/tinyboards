use crate::schema::password_resets::dsl::*;
use crate::{
    models::site::password_resets::{
        PasswordReset, PasswordResetForm,
    },
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};

impl Crud for PasswordReset {
    type Form = PasswordResetForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        password_resets.find(id_).first::<Self>(conn)
    }

    fn create(conn: &mut PgConnection, form: &PasswordResetForm) -> Result<Self, Error> {
        insert_into(password_resets)
            .values(form)
            .get_result::<Self>(conn)
    }

    fn update(
        conn: &mut PgConnection,
        id_: i32,
        form: &PasswordResetForm,
    ) -> Result<Self, Error> {
        diesel::update(password_resets.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }

    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        diesel::delete(password_resets.find(id_)).execute(conn)
    }
}
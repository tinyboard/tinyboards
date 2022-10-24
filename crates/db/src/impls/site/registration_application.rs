use crate::{
    models::site::registration_application::{RegistrationApplication, RegistrationApplicationForm},
    traits::Crud,
};
use diesel::{result::Error, dsl::*, *};
use crate::schema::registration_application::dsl::*;

impl Crud for RegistrationApplication {
    type Form = RegistrationApplicationForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        registration_application.find(id_).first::<Self>(conn)
    }

    fn create(conn: &mut PgConnection, form: &RegistrationApplicationForm) -> Result<Self, Error> {
        insert_into(registration_application)
            .values(form)
            .get_result::<Self>(conn)
    }

    fn update(conn: &mut PgConnection, id_: i32, form: &RegistrationApplicationForm) -> Result<Self, Error> {
        diesel::update(registration_application.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }

    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        diesel::delete(registration_application.find(id_)).execute(conn)
    }
}

impl RegistrationApplication {
    pub fn find_by_user_id(
        conn: &mut PgConnection,
        user_id_: i32,
    ) -> Result<Self, Error> {
        registration_application
            .filter(user_id.eq(user_id_))
            .first::<Self>(conn)
    }
}
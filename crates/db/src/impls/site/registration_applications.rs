use crate::schema::registration_applications::dsl::*;
use crate::{
    models::site::registration_applications::{
        RegistrationApplication, RegistrationApplicationForm,
    },
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};

impl Crud for RegistrationApplication {
    type Form = RegistrationApplicationForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        registration_applications.find(id_).first::<Self>(conn)
    }

    fn create(conn: &mut PgConnection, form: &RegistrationApplicationForm) -> Result<Self, Error> {
        insert_into(registration_applications)
            .values(form)
            .get_result::<Self>(conn)
    }

    fn update(
        conn: &mut PgConnection,
        id_: i32,
        form: &RegistrationApplicationForm,
    ) -> Result<Self, Error> {
        diesel::update(registration_applications.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }

    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        diesel::delete(registration_applications.find(id_)).execute(conn)
    }
}

impl RegistrationApplication {
    pub fn find_by_user_id(conn: &mut PgConnection, user_id_: i32) -> Result<Self, Error> {
        registration_applications
            .filter(user_id.eq(user_id_))
            .first::<Self>(conn)
    }
}

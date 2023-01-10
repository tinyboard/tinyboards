use crate::{
    schema::private_messages::dsl::*,
    models::user::private_messages::{PrivateMessage, PrivateMessageForm},
    traits::Crud,
};
use diesel::{result::Error, *};

impl Crud for PrivateMessage {
    type Form = PrivateMessageForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, pm_id: i32) -> Result<Self, Error> {
        private_messages.find(pm_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, pm_id: i32) -> Result<usize, Error> {
        diesel::delete(private_messages.find(pm_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &PrivateMessageForm) -> Result<Self, Error> {
        let pm = diesel::insert_into(private_messages)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(pm)
    }
    fn update(conn: &mut PgConnection, pm_id: i32, form: &PrivateMessageForm) -> Result<Self, Error> {
        diesel::update(private_messages.find(pm_id))
            .set(form)
            .get_result::<Self>(conn)
    }

}
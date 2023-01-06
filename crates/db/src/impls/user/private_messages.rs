use crate::{
    schema::dms::dsl::*,
    models::user::dms::{PrivateMessage, PrivateMessageForm},
    traits::Crud,
};
use diesel::{result::Error, *};

impl Crud for PrivateMessage {
    type Form = PrivateMessageForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, dm_id: i32) -> Result<Self, Error> {
        dms.find(dm_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, dm_id: i32) -> Result<usize, Error> {
        diesel::delete(dms.find(dm_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &PrivateMessageForm) -> Result<Self, Error> {
        let dm = diesel::insert_into(dms)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(dm)
    }
    fn update(conn: &mut PgConnection, dm_id: i32, form: &PrivateMessageForm) -> Result<Self, Error> {
        diesel::update(dms.find(dm_id))
            .set(form)
            .get_result::<Self>(conn)
    }

}
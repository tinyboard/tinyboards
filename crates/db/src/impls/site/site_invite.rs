use crate::{
    models::site::site_invite::{SiteInvite, SiteInviteForm},
    traits::Crud,
};
use diesel::{result::Error, dsl::*, *};

impl Crud for SiteInvite {
    type Form = SiteInviteForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        site_invite.find(id_).first::<Self>(conn)
    }

    fn create(conn: &mut PgConnection, form: &SiteInviteForm) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        insert_into(site_invite)
            .values(form)
            .get_result::<Self>(conn)
    }

    fn update(conn: &mut PgConnection, id_: i32, form: &SiteInviteForm) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        diesel::update(site_invite.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }

    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::site_invite::dsl::*;
        diesel::delete(site_invite.find(id_)).execute(conn)
    }
}

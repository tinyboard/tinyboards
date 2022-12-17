
use crate::{models::site::site_invite::*, traits::Crud};
use diesel::{result::Error, *};
use crate::schema::site_invite::dsl::*;

impl SiteInvite {
    pub fn read_for_token(conn: &mut PgConnection, token: &str) -> Result<Self, Error> {
        site_invite
            .filter(verification_code.eq(token))
            .first::<Self>(conn)
    }
}

impl Crud for SiteInvite {
    type Form = SiteInviteForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        site_invite.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::site_invite::dsl::*;
        diesel::delete(site_invite.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        let new = diesel::insert_into(site_invite)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::site_invite::dsl::*;
        diesel::update(site_invite.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}
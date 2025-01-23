use crate::{models::site::site_invite::*, traits::Crud, utils::{get_conn, DbPool}};
use diesel::{result::Error, *};
use crate::schema::site_invite::dsl::*;
use diesel_async::RunQueryDsl;

impl SiteInvite {
    pub async fn read_for_token(pool: &DbPool, token: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        site_invite
            .filter(verification_code.eq(token))
            .first::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for SiteInvite {
    type Form = SiteInviteForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site_invite::dsl::*;
        site_invite.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site_invite::dsl::*;
        diesel::delete(site_invite.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site_invite::dsl::*;
        let new = diesel::insert_into(site_invite)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site_invite::dsl::*;
        diesel::update(site_invite.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}
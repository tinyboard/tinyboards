use crate::{
    models::site::site::{Site, SiteForm},
    traits::Crud,
};
use diesel::{dsl::*, result::Error, *};

impl Site {
    pub fn exists(
        conn: &mut PgConnection
    ) -> Result<Self, Error> {
        use crate::schema::site::dsl::*;
        site.first::<Self>(conn)
    }
}

impl Crud for Site {
    type Form = SiteForm;
    type IdType = i32;
    
    fn read(conn: &mut PgConnection, _site_id: i32) -> Result<Self, Error> {
        use crate::schema::site::dsl::*;
        site.first::<Self>(conn)
    }

    fn create(conn: &mut PgConnection, new_site: &SiteForm) -> Result<Self, Error> {
        use crate::schema::site::dsl::*;
        let site_ = insert_into(site)
            .values(new_site)
            .get_result::<Self>(conn)?;
        
        Ok(site_)
    }

    fn update(conn: &mut PgConnection, site_id: i32, form: &SiteForm) -> Result<Self, Error> {
        use crate::schema::site::dsl::*;
        diesel::update(site.find(site_id))
            .set(form)
            .get_result::<Self>(conn)
    }

    fn delete(conn: &mut PgConnection, site_id: i32) -> Result<usize, Error> {
        use crate::schema::site::dsl::*;
        diesel::delete(site.find(site_id)).execute(conn)
    }
}


impl Site {
    pub fn read_local(conn: &mut PgConnection) -> Result<Site, Error> {
        use crate::schema::site::dsl::*;
        site.order_by(id).first::<Self>(conn)
    }

    pub fn upsert(conn: &mut PgConnection, site_form: &SiteForm) -> Result<Site, Error> {
        use crate::schema::site::dsl::*;
        insert_into(site)
            .values(site_form)
            .on_conflict(id)
            .do_update()
            .set(site_form)
            .get_result::<Self>(conn)
    }
}



use crate::{models::site::stray_images::*, traits::Crud};
use diesel::{result::Error, *};
use crate::schema::stray_images::dsl::*;
use actix_web::error::*;

impl StrayImage {
    pub async fn add_url_to_stray_images(pool: &PgPool, url: String) -> Result<Self, actix_web::Error> {
        let form = StrayImageForm { img_url: url };
        
        diesel::insert_into(stray_images)
            .values(form)
            .get_result::<Self>(conn)
            .map_err(ErrorBadRequest)
    }
}

impl Crud for StrayImage {
    type Form = StrayImageForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::stray_images::dsl::*;
        stray_images.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::stray_images::dsl::*;
        diesel::delete(stray_images.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::stray_images::dsl::*;
        let new = diesel::insert_into(stray_images)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::stray_images::dsl::*;
        diesel::update(stray_images.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}
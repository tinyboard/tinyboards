use crate::{models::site::stray_images::*, traits::Crud, utils::{DbPool, get_conn}};
use diesel::{result::Error, *};
use crate::schema::stray_images::dsl::*;
use diesel_async::RunQueryDsl;

impl StrayImage {
    pub async fn add_url_to_stray_images(pool: &DbPool, url: String) -> Result<Self, Error> {
        
        let conn = &mut get_conn(pool).await?;
        
        let form = StrayImageForm { img_url: url };
        
        diesel::insert_into(stray_images)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn remove_url_from_stray_images(pool: &DbPool, url: String) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::delete(stray_images)
            .filter(img_url.eq(url))
            .execute(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for StrayImage {
    type Form = StrayImageForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stray_images::dsl::*;
        stray_images.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stray_images::dsl::*;
        diesel::delete(stray_images.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stray_images::dsl::*;
        let new = diesel::insert_into(stray_images)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::stray_images::dsl::*;
        diesel::update(stray_images.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}
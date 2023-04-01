use crate::{models::site::uploads::*, traits::Crud, utils::{get_conn, DbPool}};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;
use crate::schema::uploads::dsl::*;

impl Upload {
    pub async fn find_by_name(pool: &DbPool, f_name: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        uploads
            .filter(file_name.eq(f_name))
            .first::<Self>(conn)
            .await
    }

    pub async fn find_by_url(pool: &DbPool, f_url: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        uploads
            .filter(upload_url.eq(f_url))
            .first::<Self>(conn)
            .await
    }
}


#[async_trait::async_trait]
impl Crud for Upload {
    type Form = UploadForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::uploads::dsl::*;
        uploads.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::uploads::dsl::*;
        diesel::delete(uploads.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::uploads::dsl::*;
        let new = diesel::insert_into(uploads)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::uploads::dsl::*;
        diesel::update(uploads.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}
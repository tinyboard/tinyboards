use crate::{
    models::site::site::{Site, SiteForm},
    traits::Crud, utils::{get_conn, DbPool}, newtypes::DbUrl,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;
use url::Url;

#[async_trait::async_trait]
impl Crud for Site {
    type Form = SiteForm;
    type IdType = i32;
    
    async fn read(pool: &DbPool, _site_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        site.first::<Self>(conn)
        .await
    }

    async fn create(pool: &DbPool, new_site: &SiteForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        let site_ = insert_into(site)
            .values(new_site)
            .get_result::<Self>(conn)
            .await?;
        
        Ok(site_)
    }

    async fn update(pool: &DbPool, site_id: i32, form: &SiteForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        diesel::update(site.find(site_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    async fn delete(pool: &DbPool, site_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        diesel::delete(site.find(site_id)).execute(conn)
        .await
    }
}

impl Site {
    pub async fn read_local(pool: &DbPool) -> Result<Site, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        site.order_by(id).first::<Self>(conn)
        .await
    }

    pub async fn upsert(pool: &DbPool, site_form: &SiteForm) -> Result<Site, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        insert_into(site)
            .values(site_form)
            .on_conflict(id)
            .do_update()
            .set(site_form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn exists(
        pool: &DbPool
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        site.first::<Self>(conn)
        .await
    }

    pub fn instance_actor_id_from_url(mut url: Url) -> Url {
        url.set_fragment(None);
        url.set_path("");
        url.set_query(None);
        url
    }

    pub async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
        use crate::schema::site::dsl::*;
        let conn = &mut get_conn(pool).await?;
        Ok(
            site
                .filter(actor_id.eq(object_id))
                .first::<Site>(conn)
                .await
                .ok()
                .map(Into::into)
        )
    }

    pub async fn read_remote_sites(pool: &DbPool) -> Result<Vec<Self>, Error> {
        use crate::schema::site::dsl::*;
        let conn = &mut get_conn(pool).await?;
        site
            .order_by(id)
            .offset(1)
            .get_results::<Self>(conn).await
    }
}
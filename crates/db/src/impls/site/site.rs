use crate::{
    models::site::site::{Site, SiteForm},
    utils::{get_conn, DbPool},
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;
use url::Url;


impl Site {
    pub async fn create(pool: &DbPool, new_site: &SiteForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        insert_into(site)
            .values(new_site)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read(pool: &DbPool) -> Result<Site, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        site.first::<Self>(conn)
        .await
    }

    pub async fn update(pool: &DbPool, form: &SiteForm) -> Result<Site, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::site::dsl::*;
        // Update the first (and only) site record
        diesel::update(site)
            .set(form)
            .get_result::<Self>(conn)
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


    pub async fn read_remote_sites(pool: &DbPool) -> Result<Vec<Self>, Error> {
        use crate::schema::site::dsl::*;
        let conn = &mut get_conn(pool).await?;
        site
            .order_by(id)
            .offset(1)
            .get_results::<Self>(conn).await
    }
}
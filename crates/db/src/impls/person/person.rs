use crate::{
    models::person::person::{Person, PersonForm},
    schema::person::dsl::*,
    traits::{Crud},
    utils::{fuzzy_search, DbPool, get_conn},
};

use diesel::{prelude::*, result::Error};
//use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl Person {
    pub async fn search_by_name(
        pool: &DbPool,
        query: &str
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::person::dsl::*;
        person.filter(name.ilike(fuzzy_search(query))).load(conn)
        .await
    }

    pub async fn update_settings(
        pool: &DbPool,
        id_: i32,
        form: &PersonForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(person.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}


#[async_trait::async_trait]
impl Crud for Person {
    type Form = PersonForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        person.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(person.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &PersonForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_person = diesel::insert_into(person)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_person)
    }
    async fn update(pool: &DbPool, id_: i32, form: &PersonForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(person.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

pub mod safe_type {
    use crate::{
        models::person::person::*,
        schema::person::*,
        traits::ToSafe,
    };

    type Columns = ( 
        id,
        name,
        display_name,
        is_banned,
        local,
        actor_id,
        creation_date,
        updated,
        avatar,
        signature,
        is_deleted,
        unban_date,
        banner,
        bio,
        inbox_url,
        shared_inbox_url,
        bot_account,
        last_refreshed_date,
    );

    impl ToSafe for PersonSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                display_name,
                is_banned,
                local,
                actor_id,
                creation_date,
                updated,
                avatar,
                signature,
                is_deleted,
                unban_date,
                banner,
                bio,
                inbox_url,
                shared_inbox_url,
                bot_account,
                last_refreshed_date,
            )
        }
    }
}
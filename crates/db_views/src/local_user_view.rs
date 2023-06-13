use crate::structs::LocalUserView;
use diesel::{/*result::Error,*/ *};
use tinyboards_db::{
    models::person::local_user::*,
    models::person::person::*,
    schema::{local_user, person, person_aggregates},
    aggregates::structs::PersonAggregates,
    //traits::{ViewToVec, ToSafe},
    utils::{get_conn, DbPool},
};

use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

type LocalUserViewTuple = (LocalUser, Person, PersonAggregates);

impl LocalUserView {

    pub async fn read(pool: &DbPool, local_user_id: i32) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let (local_user, person, counts) = local_user::table
            .find(local_user_id)
            .inner_join(person::table.on(local_user::person_id.eq(person::id)))
            .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
            .select((
                local_user::all_columns,
                person::all_columns,
                person_aggregates::all_columns,
            ))
            .first::<LocalUserViewTuple>(conn)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(Self { local_user, person, counts })
    }

    pub async fn get_by_name(pool: &DbPool, name: &str) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        
        
        let (local_user, person, counts) = local_user::table
            .inner_join(person::table.on(local_user::person_id.eq(person::id)))
            .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
            .filter(person::name.ilike(name.replace(' ', "").replace('%', "\\%").replace('_', "\\_")))
            .select((
                local_user::all_columns,
                person::all_columns,
                person_aggregates::all_columns,
            ))
            .first::<LocalUserViewTuple>(conn)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(Self { local_user, person, counts })

    }

    pub async fn get_by_email(pool: &DbPool, email_addr: &str) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        
        
        let (local_user, person, counts) = local_user::table
            .inner_join(person::table.on(local_user::person_id.eq(person::id)))
            .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
            .filter(local_user::email.ilike(email_addr.replace(' ', "").replace('%', "\\%").replace('_', "\\_")))
            .select((
                local_user::all_columns,
                person::all_columns,
                person_aggregates::all_columns,
            ))
            .first::<LocalUserViewTuple>(conn)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(Self { local_user, person, counts })
    }


}
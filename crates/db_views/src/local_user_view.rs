use crate::structs::LocalUserView;
use diesel::{result::Error, *};
use tinyboards_db::{
    models::person::local_user::*,
    schema::{local_user, person_aggregates},
    aggregates::structs::PersonAggregates,
    traits::{ViewToVec, ToSafe},
    utils::{fuzzy_search, limit_and_offset, get_conn, DbPool},
    UserSortType,
};

use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

type LocalUserViewTuple = (LocalUserSafe, PersonAggregates);

impl LocalUserView {

    pub async fn read(pool: &DbPool, local_user_id: i32) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let (local_user, counts) = local_user::table
            .find(local_user_id)
            .inner_join(person_aggregates::table.on(local_user::person_id.eq(person_aggregates::person_id)))
            .select((
                LocalUserSafe::safe_columns_tuple(),
                person_aggregates::all_columns,
            ))
            .first::<LocalUserViewTuple>(conn)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(Self { local_user, counts })
    }

    pub async fn get_from_person_id(pool: &DbPool, person_id: i32) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let (local_user, counts) = local_user::table
            .filter(local_user::person_id.eq(person_id))
            .inner_join(person_aggregates::table.on(local_user::person_id.eq(person_aggregates::person_id)))
            .select((
                LocalUserSafe::safe_columns_tuple(),
                person_aggregates::all_columns,
            ))
            .first::<LocalUserViewTuple>(conn)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok( Self { local_user, counts } )

    }

}
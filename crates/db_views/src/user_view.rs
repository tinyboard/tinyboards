use crate::local_structs::{UserView, UserSettingsView};
use diesel::{result::Error, *};
use porpl_db::{
    aggregates::structs::UserAggregates,
    schema::{
        user_,
        user_aggregates,
    },
    models::{
        user::user::{User, UserSettings, UserSafe},
    },
    traits::{ToSafe, ViewToVec},
    utils::functions::lower,
};

type UserViewTuple = (User, UserAggregates);

impl UserView {
    pub fn read(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Self, Error> {
        let (user, counts) = user_::table
            .find(user_id)
            .inner_join(user_aggregates::table)
            .select((
                user_::all_columns,
                user_aggregates::all_columns,
            ))
            .first::<UserViewTuple>(conn)?;
        
            Ok(Self {
                user,
                counts,
            })
    }

    pub fn read_from_name(
        conn: &mut PgConnection,
        name: &str,
    ) -> Result<Self, Error> {
        let (user, counts) = user_::table
        .filter(user_::name.eq(name))
        .inner_join(user_aggregates::table)
        .select((
            user_::all_columns,
            user_aggregates::all_columns,
        ))
        .first::<UserViewTuple>(conn)?;
    
        Ok(Self {
            user,
            counts,
        })       
    }

    pub fn find_by_email_or_name(
        conn: &mut PgConnection,
        name_or_email: &str,
    ) -> Result<Self, Error> {
        let (user, counts) = user_::table
            .inner_join(user_aggregates::table)
            .filter(
                lower(user_::name)
                    .eq(lower(name_or_email))
                    .or(user_::email.eq(name_or_email)),
            )
            .select((
                user_::all_columns,
                user_aggregates::all_columns,
            ))
            .first::<UserViewTuple>(conn)?;
            
            Ok(Self {
                user,
                counts,
            })    
    }

    pub fn find_by_email(
        conn: &mut PgConnection,
        from_email: &str,
    ) -> Result<Self, Error> {
        let (user, counts) = user_::table
            .inner_join(user_aggregates::table)
            .filter(user_::email.eq(from_email))
            .select((
                user_::all_columns,
                user_aggregates::all_columns,
            ))
            .first::<UserViewTuple>(conn)?;

            Ok(Self {
                user,
                counts,
            })    
    }
}

type UserSettingsViewTuple = (UserSettings, UserAggregates);

impl UserSettingsView {
    pub fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, Error> {
        let (settings, counts) = user_::table
            .find(user_id)
            .inner_join(user_aggregates::table)
            .select((
                UserSettings::safe_columns_tuple(),
                user_aggregates::all_columns,
            ))
            .first::<UserSettingsViewTuple>(conn)?;
        
        Ok(Self {
            settings,
            counts,
        })
    }
}

impl ViewToVec for UserSettingsView {
    type DbTuple = UserSettingsViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                settings: a.0,
                counts: a.1,
            })
            .collect::<Vec<Self>>()
    }
}
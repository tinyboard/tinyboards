use crate::structs::{UserSettingsView, UserView};
use diesel::{result::Error, PgConnection, *};
use tinyboards_db::{
    aggregates::structs::UserAggregates,
    models::user::user::{UserSafe, UserSettings},
    schema::{user_, user_aggregates},
    traits::{ToSafe, ViewToVec},
    utils::{functions::lower, limit_and_offset, fuzzy_search},
    UserSortType,
};
use tinyboards_utils::TinyBoardsError;

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha384;
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;

type UserViewTuple = (UserSafe, UserAggregates);

impl UserView {
    pub fn read_opt(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let user_view_tuple = user_::table
            .find(user_id)
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)
            .optional()
            .map_err(|_| TinyBoardsError::from_message("could not read user view (opt)"))?;

        Ok(user_view_tuple.map(|(user, counts)| Self { user, counts }))
    }

    pub fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, TinyBoardsError> {
        match Self::read_opt(conn, user_id) {
            Ok(opt) => match opt {
                Some(u) => Ok(u),
                None => Err(TinyBoardsError::from_message("no user view found")),
            },
            Err(e) => Err(e),
        }
    }

    pub fn from_jwt(
        conn: &mut PgConnection,
        token: String,
        master_key: String,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key).map_err(|e| TinyBoardsError::from_error_message(e, ""))?;

        let uid = claims["uid"]
            .parse::<i32>()?;

        Self::read_opt(conn, uid)
    }

    pub fn read_from_name(conn: &mut PgConnection, name: &str) -> Result<Self, Error> {
        let (user, counts) = user_::table
            .filter(user_::name.eq(name))
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)?;

        Ok(Self { user, counts })
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
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)?;

        Ok(Self { user, counts })
    }

    pub fn find_by_email(conn: &mut PgConnection, from_email: &str) -> Result<Self, Error> {
        let (user, counts) = user_::table
            .inner_join(user_aggregates::table)
            .filter(user_::email.eq(from_email))
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)?;

        Ok(Self { user, counts })
    }

    pub fn admins(conn: &mut PgConnection) -> Result<Vec<Self>, Error> {
        let admins = user_::table
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .filter(user_::admin.eq(true))
            .filter(user_::deleted.eq(false))
            .order_by(user_::published)
            .load::<UserViewTuple>(conn)?;

        Ok(Self::from_tuple_to_vec(admins))
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

        Ok(Self { settings, counts })
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

impl ViewToVec for UserView {
    type DbTuple = UserViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                user: a.0,
                counts: a.1,
            })
            .collect::<Vec<Self>>()
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct UserQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    sort: Option<UserSortType>,
    page: Option<i64>,
    limit: Option<i64>,
    search_term: Option<String>,
}

#[derive(Default, Clone)]
pub struct UserQueryResponse {
    pub users: Vec<UserView>,
    pub count: i64,
}

impl<'a> UserQuery<'a> {
    pub fn list(self) -> Result<UserQueryResponse, Error> {
        let mut query = user_::table
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .into_boxed();

        query = match self.sort.unwrap_or(UserSortType::MostRep) {
            UserSortType::New => query.then_order_by(user_::published.desc()),
            UserSortType::Old => query.then_order_by(user_::published.asc()),
            UserSortType::MostRep => query
                .then_order_by(user_aggregates::post_score.desc())
                .then_order_by(user_aggregates::comment_score.desc()),
            UserSortType::MostPosts => query.then_order_by(user_aggregates::post_count.desc()),
            UserSortType::MostComments => {
                query.then_order_by(user_aggregates::comment_count.desc())
            }
        };

        let count_query = user_::table
        .inner_join(user_aggregates::table)
        .select(UserSafe::safe_columns_tuple())
        .into_boxed();

        if let Some(search_term) = self.search_term {
            query = query.filter(user_::name.ilike(fuzzy_search(&search_term)));
        };

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(user_::deleted.eq(false))
            .filter(user_::banned.eq(false));

        let res = query.load::<UserViewTuple>(self.conn)?;

        let users = UserView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(self.conn)?;

        Ok(UserQueryResponse { users, count })
    }
}

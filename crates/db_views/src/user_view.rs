use crate::structs::{LoggedInUserView, UserSettingsView, UserView, UserMentionView, CommentReplyView};
use diesel::{result::Error, *};
use tinyboards_db::{
    aggregates::structs::UserAggregates,
    //map_to_user_sort_type,
    models::user::users::{UserSafe, UserSettings},
    schema::{user_aggregates, users},
    traits::{ToSafe, ViewToVec},
    utils::{functions::lower, fuzzy_search, limit_and_offset, get_conn, DbPool},
    UserSortType,
};
use tinyboards_utils::TinyBoardsError;

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha384;
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;
use diesel_async::RunQueryDsl;

type UserViewTuple = (UserSafe, UserAggregates);

impl UserView {
    pub async fn read_opt(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let user_view_tuple = users::table
            .find(user_id)
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(user_view_tuple.map(|(user, counts)| Self { user, counts }))
    }

    pub async fn read(pool: &DbPool, user_id: i32) -> Result<Self, TinyBoardsError> {
        match Self::read_opt(pool, user_id).await {
            Ok(opt) => match opt {
                Some(u) => Ok(u),
                None => Err(TinyBoardsError::from_message(404, "no user view found")),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn from_jwt(
        pool: &DbPool,
        token: String,
        master_key: String,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token
            .verify_with_key(&key)
            .map_err(|e| TinyBoardsError::from(e))?;

        let uid = claims["uid"].parse::<i32>()?;

        Self::read_opt(pool, uid).await
    }

    pub async fn read_from_name(pool: &DbPool, name: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (user, counts) = users::table
            .filter(users::name.eq(name))
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)
            .await?;

        Ok(Self { user, counts })
    }

    pub async fn find_by_email_or_name(
        pool: &DbPool,
        name_or_email: &str,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (user, counts) = users::table
            .inner_join(user_aggregates::table)
            .filter(
                lower(users::name)
                    .eq(lower(name_or_email))
                    .or(users::email.eq(name_or_email)),
            )
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)
            .await?;

        Ok(Self { user, counts })
    }

    pub async fn find_by_email(pool: &DbPool, from_email: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (user, counts) = users::table
            .inner_join(user_aggregates::table)
            .filter(users::email.eq(from_email))
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .first::<UserViewTuple>(conn)
            .await?;

        Ok(Self { user, counts })
    }

    pub async fn admins(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let admins = users::table
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .filter(users::is_admin.eq(true))
            .filter(users::is_deleted.eq(false))
            .order_by(users::creation_date)
            .load::<UserViewTuple>(conn)
            .await?;

        Ok(Self::from_tuple_to_vec(admins))
    }
}


impl LoggedInUserView {
    
    pub async fn read(pool: &DbPool, user_id: i32) -> Result<Self, TinyBoardsError> {

        let user_view = UserView::read(pool, user_id)
            .await    
            .map_err(|e| TinyBoardsError::from(e))?;

        let mentions = UserMentionView::get_unread_mentions(pool, user_id).await?;

        let replies = CommentReplyView::get_unread_replies(pool, user_id).await?;

        Ok( LoggedInUserView { 
            user: user_view.user, 
            counts: user_view.counts, 
            unread_notifications: mentions + replies 
        })

    }

}

type UserSettingsViewTuple = (UserSettings, UserAggregates);

impl UserSettingsView {
    pub async fn read(pool: &DbPool, user_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (settings, counts) = users::table
            .find(user_id)
            .inner_join(user_aggregates::table)
            .select((
                UserSettings::safe_columns_tuple(),
                user_aggregates::all_columns,
            ))
            .first::<UserSettingsViewTuple>(conn)
            .await?;

        Ok(Self { settings, counts })
    }

    pub async fn list_admins_with_email(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let res = users::table
            .filter(users::is_admin.eq(true))
            .filter(users::email.is_not_null())
            .inner_join(user_aggregates::table.on(users::id.eq(user_aggregates::user_id)))
            .select((
                UserSettings::safe_columns_tuple(),
                user_aggregates::all_columns,
            ))
            .load::<UserSettingsViewTuple>(conn)
            .await?;

            Ok(UserSettingsView::from_tuple_to_vec(res))
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
    pool: &'a DbPool,
    sort: Option<UserSortType>,
    page: Option<i64>,
    limit: Option<i64>,
    search_term: Option<String>,
    is_admin: Option<bool>,
    is_banned: Option<bool>,
    approved_only: Option<bool>,
}

#[derive(Default, Clone)]
pub struct UserQueryResponse {
    pub users: Vec<UserView>,
    pub count: i64,
}

impl<'a> UserQuery<'a> {
    pub async fn list(self) -> Result<UserQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        let mut query = users::table
            .inner_join(user_aggregates::table)
            .select((UserSafe::safe_columns_tuple(), user_aggregates::all_columns))
            .into_boxed();

        query = match self.sort.unwrap_or(UserSortType::MostRep) {
            UserSortType::New => query.then_order_by(users::creation_date.desc()),
            UserSortType::Old => query.then_order_by(users::creation_date.asc()),
            UserSortType::MostRep => query
                .then_order_by(user_aggregates::rep.desc()),
            UserSortType::MostPosts => query.then_order_by(user_aggregates::post_count.desc()),
            UserSortType::MostComments => {
                query.then_order_by(user_aggregates::comment_count.desc())
            }
        };

        let mut count_query = users::table
            .inner_join(user_aggregates::table)
            .select(UserSafe::safe_columns_tuple())
            .filter(users::is_deleted.eq(false))
            .filter(users::is_banned.eq(false))
            .into_boxed();

        if let Some(search_term) = self.search_term {
            query = query.filter(users::name.ilike(fuzzy_search(&search_term)));
            count_query = count_query.filter(users::name.ilike(fuzzy_search(&search_term)));
        };

        if let Some(is_banned) = self.is_banned {
            query = query.filter(users::is_banned.eq(is_banned));
            count_query = count_query.filter(users::is_banned.eq(is_banned));
        };

        if let Some(is_admin) = self.is_admin {
            query = query.filter(users::is_admin.eq(is_admin));
            count_query = count_query.filter(users::is_admin.eq(is_admin));
        };

        if self.approved_only.unwrap_or(false) {
            query = query.filter(users::is_application_accepted.eq(true));
            count_query = count_query.filter(users::is_application_accepted.eq(true));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(users::is_deleted.eq(false))
            .filter(users::is_banned.eq(false));

        let res = query.load::<UserViewTuple>(conn).await?;

        let users = UserView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(UserQueryResponse { users, count })
    }
}

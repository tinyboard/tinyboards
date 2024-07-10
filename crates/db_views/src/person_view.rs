use crate::board_view::BoardQuery;
use crate::structs::{
    CommentReplyView, LocalUserSettingsView, LoggedInUserView, PersonMentionView, PersonView,
};
use diesel::{result::Error, *};
use tinyboards_db::ListingType;
use tinyboards_db::SortType;
use tinyboards_db::{
    aggregates::structs::PersonAggregates,
    //map_to_user_sort_type,
    models::person::local_user::*,
    models::person::person::*,
    schema::{local_user, person, person_aggregates},
    traits::{ToSafe, ViewToVec},
    utils::{functions::lower, fuzzy_search, get_conn, limit_and_offset, DbPool},
    UserSortType,
};
use tinyboards_utils::TinyBoardsError;

use diesel_async::RunQueryDsl;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha384;
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;

type PersonViewTuple = (PersonSafe, Option<LocalUserSettings>, PersonAggregates);

impl PersonView {
    pub async fn read_opt(
        pool: &DbPool,
        person_id: i32,
        get_settings: bool,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        let person_view_tuple = person::table
            .find(person_id)
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .first::<PersonViewTuple>(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::from(e))?;

        if get_settings {
            Ok(person_view_tuple.map(|(person, settings, counts)| Self {
                person,
                settings,
                counts,
            }))
        } else {
            Ok(person_view_tuple.map(|(person, _, counts)| Self {
                person,
                settings: None,
                counts,
            }))
        }
    }

    pub async fn read(
        pool: &DbPool,
        person_id: i32,
        get_settings: bool,
    ) -> Result<Self, TinyBoardsError> {
        match Self::read_opt(pool, person_id, get_settings).await {
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

        Self::read_opt(pool, uid, true).await
    }

    pub async fn read_from_name(pool: &DbPool, name: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (person, settings, counts) = person::table
            .filter(
                person::name.ilike(
                    name.replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .first::<PersonViewTuple>(conn)
            .await?;

        Ok(Self {
            person,
            settings,
            counts,
        })
    }

    pub async fn find_by_email_or_name(pool: &DbPool, name_or_email: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (person, settings, counts) = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .filter(
                lower(person::name)
                    .eq(lower(name_or_email))
                    .or(local_user::email.eq(name_or_email)),
            )
            .filter(person::local.eq(true))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .first::<PersonViewTuple>(conn)
            .await?;

        Ok(Self {
            person,
            settings,
            counts,
        })
    }

    pub async fn find_by_email(pool: &DbPool, from_email: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (person, settings, counts) = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .filter(local_user::email.eq(from_email).and(person::local.eq(true)))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .first::<PersonViewTuple>(conn)
            .await?;

        Ok(Self {
            person,
            settings,
            counts,
        })
    }

    pub async fn admins(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let admins = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .filter(person::local.eq(true))
            .filter(local_user::admin_level.gt(0))
            .filter(local_user::is_deleted.eq(false))
            .order_by(person::creation_date)
            .load::<PersonViewTuple>(conn)
            .await?;

        Ok(Self::from_tuple_to_vec(admins))
    }
}

impl LoggedInUserView {
    pub async fn read(
        pool: &DbPool,
        person_id: i32,
        local_user_admin_level: i32,
    ) -> Result<Self, TinyBoardsError> {
        //let person_id = local_user_view.local_user.id;
        let person_view = PersonView::read(pool, person_id, true)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        let local_user = LocalUser::get_by_person_id(pool, person_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "No matching local user found for person.",
                )
            })?;

        let mentions = PersonMentionView::get_unread_mentions(pool, person_id).await?;

        let replies = CommentReplyView::get_unread_replies(pool, person_id).await?;

        let resp = BoardQuery::builder()
            .pool(pool)
            .user(Some(&local_user))
            .sort(Some(SortType::TopAll))
            .listing_type(Some(ListingType::Subscribed))
            .limit(Some(10))
            .build()
            .list()
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Fetching joined boards failed.")
            })?;

        let subscribed_boards = resp.boards;

        let resp = BoardQuery::builder()
            .pool(pool)
            .user(Some(&local_user))
            .sort(Some(SortType::TopAll))
            .listing_type(Some(ListingType::Moderated))
            .limit(Some(5))
            .build()
            .list()
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Fetching moderated boards failed.")
            })?;

        let moderated_boards = resp.boards;

        Ok(LoggedInUserView {
            person: person_view.person,
            settings: person_view.settings,
            counts: person_view.counts,
            unread_notifications: mentions + replies,
            admin_level: local_user_admin_level,
            subscribed_boards,
            moderated_boards,
        })
    }
}

type LocalUserSettingsViewTuple = (LocalUserSettings, PersonAggregates);

impl LocalUserSettingsView {
    pub async fn read(pool: &DbPool, person_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        let (settings, counts) = person::table
            .find(person_id)
            .inner_join(person_aggregates::table)
            .inner_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .filter(person::local.eq(true))
            .select((
                LocalUserSettings::safe_columns_tuple(),
                person_aggregates::all_columns,
            ))
            .first::<LocalUserSettingsViewTuple>(conn)
            .await?;

        Ok(Self { settings, counts })
    }

    pub async fn list_admins_with_email(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let res = local_user::table
            .filter(local_user::admin_level.gt(0))
            .filter(local_user::email.is_not_null())
            .inner_join(
                person_aggregates::table.on(local_user::person_id.eq(person_aggregates::person_id)),
            )
            .select((
                LocalUserSettings::safe_columns_tuple(),
                person_aggregates::all_columns,
            ))
            .load::<LocalUserSettingsViewTuple>(conn)
            .await?;

        Ok(LocalUserSettingsView::from_tuple_to_vec(res))
    }
}

impl ViewToVec for LocalUserSettingsView {
    type DbTuple = LocalUserSettingsViewTuple;
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

impl ViewToVec for PersonView {
    type DbTuple = PersonViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                person: a.0,
                settings: a.1,
                counts: a.2,
            })
            .collect::<Vec<Self>>()
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PersonQuery<'a> {
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
pub struct PersonQueryResponse {
    pub persons: Vec<PersonView>,
    pub count: i64,
}

impl<'a> PersonQuery<'a> {
    pub async fn list(self) -> Result<PersonQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        let mut query = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .select((
                PersonSafe::safe_columns_tuple(),
                LocalUserSettings::safe_columns_tuple().nullable(),
                person_aggregates::all_columns,
            ))
            .into_boxed();

        query = match self.sort.unwrap_or(UserSortType::MostRep) {
            UserSortType::New => query.then_order_by(person::creation_date.desc()),
            UserSortType::Old => query.then_order_by(person::creation_date.asc()),
            UserSortType::MostRep => query.then_order_by(person_aggregates::rep.desc()),
            UserSortType::MostPosts => query.then_order_by(person_aggregates::post_count.desc()),
            UserSortType::MostComments => {
                query.then_order_by(person_aggregates::comment_count.desc())
            }
        };

        let mut count_query = person::table
            .inner_join(person_aggregates::table)
            .left_join(local_user::table.on(person::id.eq(local_user::person_id)))
            .select(PersonSafe::safe_columns_tuple())
            .filter(person::is_deleted.eq(false))
            .into_boxed();

        if let Some(search_term) = self.search_term {
            query = query.filter(person::name.ilike(fuzzy_search(&search_term)));
            count_query = count_query.filter(person::name.ilike(fuzzy_search(&search_term)));
        };

        if let Some(is_banned) = self.is_banned {
            query = query.filter(person::is_banned.eq(is_banned));
            count_query = count_query.filter(person::is_banned.eq(is_banned));
        };

        if let Some(true) = self.is_admin {
            query = query.filter(person::admin_level.gt(0));
            count_query = count_query.filter(person::admin_level.gt(0));

            // order by decreasing admin level, put system acc & owner on top
            query = query.order_by(person::admin_level.desc());
        };

        if self.approved_only.unwrap_or(false) {
            query = query.filter(local_user::is_application_accepted.eq(true));
            count_query = count_query.filter(local_user::is_application_accepted.eq(true));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(person::is_deleted.eq(false));

        let res = query.load::<PersonViewTuple>(conn).await?;

        let persons = PersonView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(PersonQueryResponse { persons, count })
    }
}
